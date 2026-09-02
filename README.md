# dpsi

[![crates.io](https://img.shields.io/crates/v/dpsi.svg)](https://crates.io/crates/dpsi)
![CI](https://github.com/msmorgan/dpsi/actions/workflows/ci.yml/badge.svg)

The dumbest possible string interner, pronounced "Dipsy".

`Ident` is a `Copy`, lifetime-free handle to an interned string. Equality is a
single pointer comparison, so it costs the same whether the string is four
bytes or four hundred. Ordering compares the text instead, so sorted output is
stable from run to run.

- 100% safe Rust. The crate is `#![forbid(unsafe_code)]`.
- Zero dependencies by default. `serde` is optional, and it does not enable
  `derive`.
- Readable in a minute. The pool is a `HashSet<&'static str>` behind an
  `RwLock`, and interning is a `Box::leak`. There is nothing else.

## Example

```rust
use std::collections::HashMap;

use dpsi::Ident;

let from_literal = Ident::new("Forest");
let from_owned = Ident::new(&String::from("Forest"));

// Equal text interns to one allocation, so `==` compares addresses.
assert_eq!(from_literal, from_owned);
assert!(std::ptr::eq(from_literal.as_str(), from_owned.as_str()));

// The handle carries no lifetime, and the text behind it is `'static`.
let text: &'static str = from_literal.as_str();
assert_eq!(text, "Forest");

// `Ident` hashes its text and borrows as `str`, so a `&str` probes the map.
let mut counts: HashMap<Ident, u32> = HashMap::new();
counts.insert(from_literal, 1);
assert_eq!(counts.get("Forest"), Some(&1));
```

## Memory: interning leaks

**Every string this crate interns is leaked for the lifetime of the process.**
`Ident::new` allocates a copy of text it has not seen before, leaks it, and
adds it to a process-wide pool. Nothing is ever freed. The pool only grows.

That is the whole trade, not an oversight. The leak is what buys the `'static`
lifetime, the `Copy` handle, and the pointer equality.

**Interning attacker-controlled input grows memory without bound.** Every
distinct string costs an allocation that is never returned, so a caller who
can choose the text can choose how much memory the process holds. Usernames,
request paths, query parameters, log lines, parsed file contents from an
untrusted source: do not pass these to `Ident::new`.

Intern bounded vocabularies. Keywords, opcodes, field names, card names,
attribute names — sets whose size is fixed by your program rather than by its
input.

To ask whether some text is already interned without interning it, use
`Ident::get`. It takes a read lock, looks the text up, and never inserts, so a
miss costs nothing and leaks nothing.

```rust
use dpsi::Ident;

let known = Ident::new("Forest");

assert_eq!(Ident::get("Forest"), Some(known));

// A miss returns `None` instead of growing the pool.
assert_eq!(Ident::get("not-in-the-pool"), None);
assert_eq!(Ident::get("not-in-the-pool"), None);
```

## Hashing: content or address

`Ident` hashes its text. That is what makes `Borrow<str>` work, and it is why
`HashMap<Ident, _>::get("literal")` in the example above compiles at all: you
can probe a map with a plain `&str` and never build an `Ident`. The cost is
that every probe reads the whole string.

`ByAddr` wraps an `Ident` and hashes the interned allocation's address
instead, which is constant work no matter how long the string is. What it
gives up is `Borrow<str>`: a `&str` does not know the interned address, so you
have to build the key first. Build it with `Ident::get`, not `Ident::new` — a
lookup should not intern the text it fails to find.

How much this actually buys is unmeasured. For the short identifiers this
crate is built for, hashing a handful of bytes is already cheap, so expect a
modest win at best; a benchmark ticket will replace this paragraph with
numbers, and may well conclude the win is not worth reaching for.

Two caveats come with hashing addresses. Iteration order of a `ByAddr`-keyed
map is not reproducible across runs, even under a deterministic hasher,
because the input is an address and addresses depend on allocation order and
on the allocator; sort the entries if you need stable output, since `Ord` on
`ByAddr` compares text just as it does on `Ident`. And addresses are aligned,
so their low bits are always zero: any hasher that mixes handles them fine,
but an identity hasher such as `nohash` will pile every key into a fraction of
the buckets. Do not pair `ByAddr` with one.

```rust
use std::collections::HashMap;

use dpsi::ByAddr;
use dpsi::Ident;

let mut arity: HashMap<ByAddr, u32> = HashMap::new();
arity.insert(ByAddr(Ident::new("cons")), 2);

// Probe with `Ident::get`, which returns `None` instead of interning.
let key = Ident::get("cons").map(ByAddr);
assert_eq!(key.and_then(|key| arity.get(&key)), Some(&2));

assert!(Ident::get("never-interned").is_none());
```

## Serde

The optional `serde` feature adds `Serialize` and `Deserialize` for `Ident`
and `ByAddr`. It does not enable serde's `derive`; every impl is hand-written,
so turning the feature on pulls in nothing beyond `serde` itself.

```toml
[dependencies]
dpsi = { version = "0.1", features = ["serde"] }
```

The wire form is just the text. An `Ident` is interchangeable with a `String`
or a `&str` in any format, and a value read back is pointer-equal to
`Ident::new` of the same text. Deserializing interns, so everything in the
memory section applies to whatever you deserialize.

```rust
// Compiled either way; the body runs only with the `serde` feature on.
#[cfg(feature = "serde")]
{
    use dpsi::Ident;

    let json = serde_json::to_string(&Ident::new("Forest")).unwrap();
    assert_eq!(json, r#""Forest""#);

    let back: Ident = serde_json::from_str(&json).unwrap();
    assert!(std::ptr::eq(back.as_str(), Ident::new("Forest").as_str()));
}
```

Some formats also have an *identifier position* — enum variant names, and map
keys in formats that distinguish them — where the token is not a quoted
string. `IdentSeed` reads those, going through `deserialize_identifier` rather
than `deserialize_str`. Pass it to `EnumAccess::variant_seed` in a
hand-written `Deserialize` to capture a variant name you do not know ahead of
time.

```rust
#[cfg(feature = "serde")]
{
    use dpsi::Ident;
    use dpsi::IdentSeed;
    use serde::de::DeserializeSeed;

    let mut de = serde_json::Deserializer::from_str(r#""Forest""#);
    let ident = IdentSeed.deserialize(&mut de).unwrap();

    assert_eq!(ident, Ident::new("Forest"));
}
```

## Alternatives

Every cell below was read from the named crate's current published
documentation and source on 2026-09-02; the versions checked are in the first
column. "Cached hash" means the handle stores a precomputed hash of the text,
so hashing the handle never touches the bytes.

| Crate | unsafe | Deps | `&'static str` | `Copy` | Lifetime on handle | Cached hash | `no_std` |
|---|---|---|---|---|---|---|---|
| `dpsi` 0.1 (`Ident`) | no | none | yes, `Ident::as_str` | yes | no | no, hashes the text | no |
| `ustr` 1.1 (`Ustr`) | yes | 4 | yes, `Ustr::as_str` | yes | no | yes, `Ustr::precomputed_hash` | no |
| `internment` 0.8 (`Intern`) | yes | 1 | yes, `Intern::as_ref` | yes | no | no, hashes the pointer | no |
| `symbol_table` 0.5 (`GlobalSymbol`) | yes | 3 | yes, `GlobalSymbol::as_str` | yes | no | no, hashes a `u32` index | no |
| `lasso` 0.7 (`Spur`) | yes | 1 | no, `resolve` borrows the interner | yes | no | no, hashes an integer key | yes, `no-std` feature |

Notes on the rows:

- Dependency counts are non-optional dependencies only, ignoring dev- and
  optional dependencies: `ustr` pulls `ahash`, `byteorder`, `lazy_static` and
  `parking_lot`; `internment` and `lasso` pull `hashbrown`; `symbol_table`
  pulls `crossbeam-utils`, `foldhash` and `hashbrown`. `dpsi` pulls nothing
  unless you turn on `serde`.
- `internment` also offers `ArcIntern`, which refcounts and frees but is not
  `Copy`, and `ArenaIntern<'a, T>`, which does carry a lifetime. The row
  describes `Intern`, the leak-forever one.
- `symbol_table` also offers a non-global `SymbolTable` whose `Symbol` needs
  the table to resolve. The row describes `GlobalSymbol`.
- `lasso` keys are integer indices into an interner you own, which is why they
  resolve to a borrowed `&str` rather than a `'static` one. In exchange it is
  the only one here that can free, that runs on `no_std`, and that lets you
  pick an 8-, 16-, 32-, or pointer-sized key.
- `dpsi` also offers `ByAddr`, which hashes the address rather than the text.
  It is still not a cached hash; nothing is precomputed.

If you need any of the things in that table that `dpsi` says "no" to, use one
of the others. This crate is the smallest thing that works.

## Non-goals

- **`no_std`.** It would add two dependencies, a pool trait, and an allocator
  hook to a crate whose entire pitch is zero dependencies and one screen of
  code, and no one has asked for it. The design is written down and deferred
  in `docs/tickets/maybe/no-std-feature.md`, along with why a spinlock-backed
  pool is a bad fit for the targets that would want it.
- **Pool customization.** A public pool would turn the canonical-pointer
  invariant into a contract downstream code can violate, which silently breaks
  `Eq`. The pool stays private.
- **Freeing interned strings.** Refcounting or arenas would take back the
  `Copy` handle and the `'static` lifetime, which are the only reasons to
  reach for this crate. Use `lasso` or `internment`'s `ArcIntern` instead.

## Contributing

`scripts/check` is the gate. It runs `cargo fmt --check`, clippy at
`-D warnings` in both feature configurations, the tests in both, `cargo doc`
with `-D warnings`, and a `cargo package --list` check that repo scaffolding
stays out of the published crate. It has to be green before anything lands.

Work is tracked as files: one ticket per unit of work under `docs/tickets/`,
whose folder is its status. `docs/tickets/README.md` explains the layout.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
