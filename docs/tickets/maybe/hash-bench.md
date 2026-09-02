---
needs: [by-addr-wrapper]
---
**Measure `Ident` vs `ByAddr` vs `&str` map-lookup cost so the README can
state a number instead of "modest".**

Criterion bench (dev-dependency only) over `HashMap` lookups keyed by `&str`,
`Ident`, and `ByAddr`, at 8-, 32-, and 128-byte keys, with the default SipHash
and with `rustc-hash`'s Fx. Report the ratio, not absolute times.

Outcome: update the README's hash-tradeoff section with the measured range and
the key length at which `ByAddr` starts to matter. If the gain is negligible
across the board, say that in the README instead; the wrapper still earns its
place for the `Borrow`-free semantics.
