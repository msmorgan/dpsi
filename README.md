# dpsi

The dumbest possible string interner, pronounced "Dipsy".

A `Copy`, lifetime-free `Ident(&'static str)` with O(1) pointer equality,
written in 100% safe Rust with zero dependencies by default. Interned strings
are leaked for the lifetime of the process by design.

This README is a stub; the real one lands with the `docs-and-readme` ticket.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
