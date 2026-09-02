---
needs: [ident-core]
---
**`ByAddr`: an `Ident` wrapper that hashes by address.**

`Ident`'s derived `Hash` rehashes the string bytes on every map lookup, which
is the one place `ustr`'s cached hash beats it. `ByAddr` trades `Borrow<str>`
for an O(1) hash.

Decided in design: this is a **wrapper**, not a second `Ident` type in a
`by_addr` module. One interner, one invariant; the map key is chosen at the
`HashMap` declaration. Shape follows the `by_address` crate idiom.

`pub struct ByAddr(pub Ident)` in `src/by_addr.rs`, re-exported at the root.

- `Hash`: hash `self.0.as_str().as_ptr() as usize`.
- `PartialEq`/`Eq`: pointer equality (delegate to `Ident`).
- `PartialOrd`/`Ord`: content order (delegate to `Ident`), never address.
- `Deref<Target = Ident>`, `From<Ident> for ByAddr`, `From<ByAddr> for Ident`,
  `Display`, `Debug`, `Copy`, `Clone`, `Default`.
- Under `serde`: `Serialize`/`Deserialize` pass straight through to `Ident`.

Docs must cover:

- when to reach for it (hot map lookups, long keys) and that the gain is
  modest for short identifiers until `hash-bench` says otherwise
- what you lose: no `Borrow<str>`, so probe maps with `Ident::get` rather than
  `Ident::new` to avoid interning untrusted input
- iteration order of a `ByAddr`-keyed map varies across runs even with a
  deterministic hasher such as Fx, because addresses do
- addresses have zero low bits from alignment; SipHash and Fx are fine,
  identity hashers such as `nohash` will cluster

Tests: `HashMap<ByAddr, _>` insert and lookup round-trip; two `ByAddr` from
equal text produce equal hashes; `Ord` matches `Ident`'s.
