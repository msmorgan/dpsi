---
needs: [ident-core]
---
**Optional `serde` feature: `Serialize`, `Deserialize`, and `IdentSeed`.**

Port the serde half of the deckmaste.rs reference
(`/home/msmorgan/Projects/deckmaste.rs/default/crates/dpsi/src/lib.rs`) behind
`#[cfg(feature = "serde")]`. The `serde` dependency is optional and does not
enable `derive`: write the impls by hand so the crate pulls in nothing extra.

- `Serialize`: serialize as the string (what `#[serde(transparent)]` did).
- `Deserialize`: `deserialize_str` through a visitor whose `visit_str` interns.
- `IdentSeed`: a `DeserializeSeed` that goes through `deserialize_identifier`,
  for reading bare tokens (enum variant names, map keys in identifier
  position) as `Ident`s. deckmaste.rs's `macro_ron` uses it with
  `EnumAccess::variant_seed`; keep the name and semantics so the switchover is
  a path change.

Dev-dependencies: `serde_json`, and `serde_test` if it makes the identifier
path easier to exercise.

Tests:

- string round-trip through `serde_json`
- `IdentSeed` reads a bare variant token via `EnumAccess::variant_seed`
- a deserialized value is pointer-equal to `Ident::new` of the same text
- `cargo test --no-default-features` still passes (nothing serde-shaped leaks
  out from behind the gate)
