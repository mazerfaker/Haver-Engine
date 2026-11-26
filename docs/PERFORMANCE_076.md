# Forge Engine v0.7.6 — Performance report

## Scope and verification
Based on the actual v0.7.5 source tree. No Auto Rig or external animator. Existing examples retained unchanged. Rust/Cargo unavailable in packaging environment; no compilation, runtime benchmarks, Windows EXE or GPU tests performed.

## Changes
- Viewport: bounded background image decode queue (8 jobs and 8 results), at most 4 uploads per poll, metadata-based invalidation, failed decode tracking, release unused cached handles. The fallback and asset browser still decode synchronously. Memory is not globally bounded by bytes.
- Sprite lookup: hash index and membership set replace repeated linear searches in native viewport; sorting and light calculations remain.
- Avoid marking render targets changed when resolution is unchanged; inactive game camera does not resize.
- Reuse editor preview particle entities; limit preview to 4096. CPU simulation remains.
- Asset scan interval 2 -> 5 seconds. Remove per-frame JSON serialization from status bar.
- Build profiles: development (opt-level 1), performance (opt-level 3, one codegen unit), lightweight (opt-level z, fat LTO, stripped symbols). Default release retains thin LTO and strips debug info. No dependency pruning: compatibility prioritized. Profiles may trade compile time, size and speed differently; measure before choosing.
- Headless benchmark: 60 warm-up + 600 measured fixed steps, mean/p95/max simulation duration and scene load duration. Not GUI startup, rendered FPS or GPU performance. Build script records EXE file lengths.

## Reproduce on Windows
Run CHECK_WINDOWS.bat, then BUILD_WINDOWS.bat or BUILD_PERFORMANCE.bat / BUILD_LIGHTWEIGHT.bat / BUILD_DEVELOPMENT.bat. Each profile writes the same dist and portable ZIP; archive results separately. Run BENCHMARK.bat after each build and save benchmark.json separately. Compare same scene, machine, power mode and profile; repeat at least five times. Existing editor profiler is retained. Development executable may open a console. Release binaries and logs are not supplied here.

## Remaining work
Full docking and icon system, GPU lighting/normal maps, GPU particle simulation, audio preview/effects, visual UI designer, Signal Lost Deluxe combat/boss/new levels, physics-specific optimization, dependency feature gating, resource compression, GUI startup measurement, global RAM/VRAM budget and full asynchronous asset browser are NOT implemented in this update. Existing functionality retained at source level only; regression test execution pending.
