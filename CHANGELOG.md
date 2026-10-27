# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.1 - 2026-10-27

### Changed

- `Ident`'s `PartialEq` implementation now compares thin data pointers
  (`as_ptr()`) rather than fat `*const str` pointers, saving the redundant
  length comparison.

## 0.1.0 - 2026-09-02

### Added

- `Ident`, a `Copy`, lifetime-free handle to an interned string. Equality is a
  single pointer comparison; ordering compares the text, so sorted output is
  stable across runs. Interned strings are leaked for the lifetime of the
  process.
- `Ident::get`, which looks text up in the pool without interning it, so
  untrusted input can be probed without growing the pool.
- `ByAddr`, an `Ident` wrapper that hashes the interned allocation's address
  instead of its text. Constant-time hashing in exchange for `Borrow<str>`.
- Optional `serde` feature: `Serialize` and `Deserialize` for `Ident` and
  `ByAddr`, with the text as the wire form, plus `IdentSeed` for reading an
  `Ident` from an identifier position such as an enum variant name. The
  dependency is optional and does not enable `derive`.
