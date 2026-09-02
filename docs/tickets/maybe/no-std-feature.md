---
needs: [ident-core]
design: true
---
**Deferred: `no_std + alloc` support. Recorded so the design isn't lost. Opens
only when a real user asks for it.**

The original design (from the deckmaste.rs handoff) was:

- `#![cfg_attr(not(feature = "std"), no_std)]` with `extern crate alloc`.
- `std` on by default. With `std` off, `hashbrown` replaces `HashSet` and
  `spin` replaces `RwLock`, both as optional dependencies gated on the absence
  of `std`.
- `Box::leak` still works under `alloc`.
- A further idea for heapless targets: an init hook accepting a bump
  allocator over a `static` byte buffer with an `AtomicUsize` cursor, giving
  `'static` slices without a heap.
- Pool customization via `OnceLock` so firmware could install its own pool.

Why it was cut from 0.1 (decided 2026-09-02):

- The crate's differentiator against `ustr` and `internment` is that it is
  auditable in one screen with zero dependencies and zero unsafe. `no_std`
  adds two dependencies, a pool trait, and an allocator hook to a crate named
  "dumbest possible".
- No identified audience. Compilers, linters, macro engines, and CLI tools all
  have `std`. Firmware with bounded RAM is the worst fit for leak-forever
  interning; a fixed string set there is better served by literals or `phf`.
- A public pool trait turns the canonical-pointer invariant from a private
  guarantee into a contract downstream code can violate, silently breaking
  `Eq`. `OnceLock` override also has an init-order footgun: the first
  `Ident::new` locks in the default pool before the override runs.
- Spinlocks deadlock when an interrupt handler interns while the lock is held
  on single-core targets, which is exactly where `no_std` lives.

Why deferring is free: the intern function is private, so a later `no_std`
feature swaps the backing store without touching the public API. If this
opens, keep `std` the default and keep the pool private; do not add the
customization hook without a concrete user.
