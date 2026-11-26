# Haver Engine v0.7.42 — Engine Stabilization

Base: 352e737bae9375e748ff432f053c13d4ec0f92ed (Windows CI #3 passed). Candidate until its own CI and manual checks complete.

- Explicit f32 stroke widths in the editor, removed unused test import. The editor shared Player DSP module intentionally retains Player-only functions; dead_code allowance limited to that module, no APIs removed.
- Preference sidecars reject non-files and oversized serialized output; four Rust regression tests exercise replacement, clamping, rejection and output budget.
- CI pins the verified Rust 1.100.0 toolchain, caches pip dependencies using pinned Pillow requirements, fetches only Windows target and preserves Rust caching.
- Cargo.lock changes only the root package version; dependencies and internal crate name preserved. Source/update export uses exact verified v0.7.41 base; update excludes Cargo.lock and target.
- No Auto Rig or automatic animator; old engine systems and roadmap retained.

Manual GUI, GPU/audio playback and performance validation remain required. Compilation and tests for v0.7.42 are pending.
