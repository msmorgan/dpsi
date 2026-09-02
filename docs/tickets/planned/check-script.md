---
needs: []
---
**`scripts/check`: the one command that has to be green before integrate.**

A fish script (this machine's shell; keep it POSIX-portable enough to port
later) that runs, stopping at the first failure:

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo package --list
```

The last step is a sanity check that `.workspaces/`, `docs/tickets/`, and
`kata.toml` are excluded (see `Cargo.toml`'s `exclude`); fail if any of them
appear.

Mention the script in the README's contributing section once `docs-and-readme`
lands; until then a comment at the top of the script is enough.
