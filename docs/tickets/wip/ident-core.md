---
needs: []
---
**Port `Ident` from deckmaste.rs and add ordering plus a non-inserting lookup.**

Reference implementation:
`/home/msmorgan/Projects/deckmaste.rs/default/crates/dpsi/src/lib.rs`. Port
everything except the serde code (that is `serde-feature`).

Keep the shape exactly: `pub struct Ident(&'static str)` with a private field,
so every value is produced by the interner and pointer equality is content
equality. The pool stays a `LazyLock<RwLock<HashSet<&'static str>>>` with the
read-then-write fast path and `Box::leak` for new entries. No pool
customization, no `OnceLock` override, no `no_std` (see `no-std-feature` in
`maybe/` for why those were cut).

Trait set, all required: `Copy`, `Clone`, `Debug`, `Eq`, `Hash`, `Default`,
`Deref<Target = str>`, `AsRef<str>`, `Borrow<str>`, `Display`, `From<&str>`,
`PartialEq<str>`, `PartialEq<&str>`.

Decisions already made:

- `PartialEq` is `std::ptr::eq` on the inner pointer. `Hash` stays derived
  (content-based) so `HashMap<Ident, _>::get("literal")` works through
  `Borrow<str>`. Keep the
  `#[expect(clippy::derived_hash_with_manual_eq, reason = ...)]` and the
  comment explaining why the pair is coherent.
- **Do not derive `PartialOrd`/`Ord`.** Deriving would compare addresses,
  which vary run to run. Implement both by
  `self.as_str().cmp(other.as_str())`.
- Add `Ident::get(s: &str) -> Option<Ident>`: read-lock lookup that never
  inserts. This is what callers use to probe with untrusted input without
  growing the pool, and it is what `ByAddr` maps need once `Borrow<str>` is
  gone.
- `Default` goes through the interner (`Ident::new("")`) so the empty string
  is canonical too.
- `#![forbid(unsafe_code)]` stays.

Document every public item. The crate-level doc is the docs ticket's job, but
the type doc must state the canonical-pointer invariant and the leak.

Tests (unit, in `src/lib.rs`):

- equal text from a literal and from a `String` shares one allocation
- an 8-thread race on the same text yields one pointer
- `Ord` is content order and does not depend on interning order
- `get` returns `None` for never-interned text and does not insert it
- `Default` is pointer-equal to `Ident::new("")`
- `HashMap<Ident, _>::get("literal")` finds the entry
