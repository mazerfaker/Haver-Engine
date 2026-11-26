# Forge Engine v0.7.15 — Stability & Regression

## Evidence from the Windows v0.7.14 log
- Rust 1.100.0, target x86_64-pc-windows-msvc.
- Full and minimal-player Cargo checks succeeded. 29 library + 5 editor unit tests passed (34 total).
- Debug binaries built. First three player smoke and editor-model QA checks succeeded.
- Level 4 failed to deserialize: missing field `looping` at line 2047 column 7. Release packaging did not run.
- 30 distinct editor warnings: 26 f32 fallback diagnostics, three unused functions, one diagnostic covering two unused fields. Minimal build emitted three library warnings and two player warnings.

## Source changes
- All 26 reported literal locations explicitly typed as f32, including both window dimensions.
- Moved viewport cache counters into the active Assets & performance window; removed the unused duplicate profiler/asset panel functions and two unused editor fields.
- Gated Rapier-only imports, mutable Lua spawn buffer and audio-only cue/mutable HUD binding by their corresponding Cargo features. No global warning suppression.
- Bundled level 4 hotfix. Missing animation `looping` now defaults to true (the existing Animation default); explicit false is retained, invalid field types are still rejected. No other animation field becomes optional.
- Three new Rust regression tests: all four actual scene files through Project::load + roundtrip, missing/false/invalid looping, and legacy named clip without looping. Tests are written, not yet executed here.
- Windows build now includes minimal library tests, consistent UTF-8 logging, profile-specific ZIP names, build-info.json and preserves the verified marker during CheckOnly. When upgrading with a v0.7.14 Cargo.lock, only its local root package version is migrated; dependency declarations remain unchanged.
- GPU_CHECK.bat checks the opt-in lab Rust target; GPU_LIGHTING_LAB.bat is still required for graphical WGSL validation. Main renderer and CPU particle fallback are unchanged.
- BENCHMARK_ALL.bat records four simulation reports and build metadata. Existing Lightweight profile remains available; no measured EXE shrink or FPS gain is claimed.

## Windows verification
1. Keep your v0.7.14 folder as a backup. Extract the full source or overlay the update ZIP onto a copy of the v0.7.14 folder. Keep Cargo.lock and target if present, do not overwrite personal project files.
2. Run BUILD_WINDOWS.bat. This executes full and minimal checks/tests, debug builds, four player and editor-model checks, release build, staged release smoke tests and portable packaging. The standard release ZIP is ForgeEngine-v0.7.15-Windows-portable.zip.
3. On success run dist/START_EDITOR.bat and dist/SignalLost/PLAY.bat. GUI launch and actual gameplay must be verified manually.
4. Run GPU_CHECK.bat then GPU_LIGHTING_LAB.bat. A successful Rust check alone does not establish WGSL correctness. Send forge-gpu-check.log and console output if the lab fails.
5. Optionally run BUILD_LIGHTWEIGHT.bat, compare build-info.json for the two profiles, and run BENCHMARK_ALL.bat on the same hardware. Save the standard release measurements first; dist reflects the most recent successful build.

## Limits
Rust/Cargo and PowerShell are unavailable in the packaging environment. No new Rust compilation, test execution, Windows EXE, GUI/WGSL run, or performance measurement is claimed. Existing GPU-runtime integration, arbitrary split docking and audio effects remain unfinished. No Auto Rig or automatic animator integration.
