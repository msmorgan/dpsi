---
needs: [docs-and-readme, check-script]
---
**Get 0.1.0 ready for `cargo publish`. The publish itself is the user's action.**

- Review `Cargo.toml` metadata: description, `repository`/`homepage` once a
  remote exists, `keywords`, `categories`, `readme`, `license`,
  `rust-version`, `exclude`, `[package.metadata.docs.rs] all-features = true`.
- `scripts/check` green.
- `cargo package --list` reviewed by eye: nothing but the crate.
- `cargo publish --dry-run --all-features` succeeds.
- Fill in the 0.1.0 date in `CHANGELOG.md`.
- Run the README doctests one more time on a clean `target/`.

Then stop and hand the actual `cargo publish` to the user (it needs their
crates.io token).

Post-publish follow-up, recorded here so it isn't forgotten but **not this
repo's work**: in deckmaste.rs, point `deckmaste_core` and `macro_ron` at the
crates.io `dpsi`, delete `crates/dpsi`, and drop it from the workspace members.
That is a deckmaste.rs ticket.
