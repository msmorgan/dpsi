---
needs: [serde-feature, by-addr-wrapper]
---
**Crate docs and README, one source, with doctests. Plus `CHANGELOG.md`.**

Use `#![doc = include_str!("../README.md")]` so the README and the crate-level
rustdoc are the same text and the examples are doctests.

Contents, in roughly this order:

1. What it is: the dumbest possible string interner, pronounced "Dipsy". A
   `Copy`, lifetime-free `Ident(&'static str)` with O(1) pointer equality.
2. The one-screen pitch: 100% safe Rust, zero dependencies by default,
   readable in a minute. Quick example.
3. **Prominent** memory note: interned strings are leaked for the process
   lifetime by design. Interning attacker-controlled input grows memory without
   bound; use `Ident::get` to probe without inserting.
4. Hash-cost tradeoff: `Ident` hashes content so `Borrow<str>` lookups work;
   `ByAddr` hashes the address for hot lookups and gives up `Borrow<str>`.
   State the determinism and identity-hasher caveats. Keep the speed claim
   hedged until `hash-bench` produces numbers.
5. Serde: the `serde` feature, string round-trip, and `IdentSeed` for
   identifier-position tokens with a short example.
6. Comparison table against `ustr`, `internment`, `symbol_table`, and `lasso`:
   unsafe, dependencies, `&'static str` access, `Copy`, cached hash,
   lifetimes. Check each crate's current docs before filling a cell; do not
   write from memory.
7. Non-goals: `no_std`, pool customization, freeing interned strings. One line
   each on why, pointing at `docs/tickets/maybe/no-std-feature.md` for the
   deferred design.
8. Contributing: `scripts/check`, tickets under `docs/tickets/`.

`CHANGELOG.md` with a 0.1.0 entry (Keep a Changelog shape, date filled in by
`publish-checklist`).
