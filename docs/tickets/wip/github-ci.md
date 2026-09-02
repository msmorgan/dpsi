---
needs: [check-script]
---
**GitHub Actions workflow running `scripts/check` on stable and on the MSRV
(`rust-version` in `Cargo.toml`).**

`.github/workflows/ci.yml`, triggered on push and pull request. Matrix:
`stable` and `1.85`. Install fish on the runner
(`apt-get install -y fish` on ubuntu-latest is enough) so the script runs
as-is; do not fork a POSIX copy. Cache `~/.cargo` and `target/` with the usual
`Swatinem/rust-cache` action.

The workflow can be written and committed before a remote exists; it just
won't run until the repo is pushed. Add the CI badge to the README when the
remote's URL is known (`publish-checklist` fills in `repository` at the same
time).
