# Forge Engine v0.7.37 — Textured Animation & Cascaded Audio

Source candidate derived from the exact v0.7.36 archive. No Auto Rig or automatic animator. No added dependencies.

## Implemented
- Animation: synchronized preview uses existing egui texture handle, atlas UVs, sampled position/rotation, runtime frame geometry/pivot, visual scale, tint and opacity. Fallback shape when texture is unavailable; preview is clipped to its canvas and uses fixed fit based on base size. No Lua events dispatched. Correct clip-relative displayed frame number.
- Audio: optional top-level custom bus `cascade_runtime` (off by default). The compatible PCM16 sources in its subtree are mixed through 2–8 independent DSP stages, child before parent, with atomic live effect/gain updates and smoothed gains. Master and built-in parent applied at sink once. 2–16 sources, existing 32 MiB PCM budget, identical rate/channels/loop mode, equal lengths for loops; spatial/source DSP/speed differences fall back to legacy playback. Existing direct summed mode unchanged. Routing topology is fixed at scene load; restart scene for changes. Each stage clamps output, so extreme peaks may clip. Finite tail limited to 2 seconds. This is not a general multi-format mixer. Added RUN_CASCADE_BUS_SCENE.bat, test with F8. Unverified Rust/GPU/audio execution.
- Asset database: reverse transitive owner lookup, explicit button in resource detail, atomic cache replacement with temporary file. Existing cache stamps and manual refresh semantics unchanged. No resource deletion; dynamic Lua references may be missed.
- Prefab: inline nested hierarchy editor can change external child attachment and offsets; existing transactional save/cycle checks and backup retained. No cross-file object transfer.
- Lighting/performance: render-world-only normal-map assets (intended to avoid retaining CPU copies after extraction), ambient-only shader early return when no active lights. No measured FPS/RAM/VRAM savings. Existing soft-shadow algorithm retained. GPU shader and asset lifetime must be tested.
- QA: Python static/fixture tests, including 4 new tests; new Rust unit tests for cascade signal and upstream dependencies. Cargo unavailable, Rust compilation/test execution not verified. Existing prior regressions not certified fixed.

## Installation
Extract flat update archive over v0.7.36 (retain Cargo.lock and target); prepare_lock.ps1 migrates only root package version. Run QUICK_CHECK.bat, QUICK_TEST.bat, then QUICK_BUILD.bat. Test RUN_CASCADE_BUS_SCENE.bat and RUN_GPU_SCENE.bat, check original sprite preview in Animation Studio.
