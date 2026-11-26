# Forge Engine v0.7.14

Source-only maintenance release, based on v0.7.13. Native Rust/Bevy/egui retained. No Auto Rig.

## Implemented (source changes, not compiled here)
- Asset Browser pagination, four thumbnail decode attempts per frame, page-scoped thumbnail retention. This is a thumbnail limit, not a global RAM/VRAM limit. Decoding is still synchronous for thumbnails.
- Nested HUD container paths using `/`, up to eight levels, rendered as collapsible subgroups in Player. Existing flat groups and ungrouped widgets remain supported. No inventory data model or visual tree editor added.
- Mixer indexes source objects once per update and skips volume changes within 0.001 (except mute). No stereo positioning, EQ or reverb added.
- Explicit Save dock button writes existing dock settings. Floating panel settings continue to use their separate existing save action. No arbitrary tab splitting.
- Windows build names updated and CI feature-matrix transcript included in artifacts.

## Not completed
GPU lighting runtime integration, GPU particle runtime, true arbitrary docking, new design overhaul, audio effects, verified Windows EXE. Existing GPU lab and CPU rendering remain unchanged. No measured startup/FPS/EXE gains.

## Verification
Run `BUILD_WINDOWS.bat` and `QA_MATRIX.bat` on Windows with Rust MSVC and C++ Build Tools. Send forge-check.log and forge-matrix.log (CI) if failures occur. See QA_STATIC_0714.txt for checks performed here.
