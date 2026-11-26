# Forge Engine 0.7.13 — Professional Editor & Engine

Source-only incremental release from the exact 0.7.12 archive. No Auto Rig. Previous demo files unchanged.

## Editor
- Panel title bars act as drag sources for existing docking targets. Added Top target; left/right/bottom/float remain. Existing layout persistence is unchanged (Studio > Save workspace and Panels > Save layout). No arbitrary tab tree or nested split docking yet.
- Three resolution-independent line icons and compact theme spacing. Old theme choices preserved.
- CPU viewport skips non-selected, non-camera objects outside expanded viewport bounds (rotation-aware AABB). Native Bevy renderer unchanged.

## Audio
- `Sound.spatial` defaults false; `distance` defaults 400. Enable Distance attenuation (SFX) in Inspector. Standalone mixer attenuates positional SFX linearly to zero at range; player object is listener, camera fallback. Music is unaffected. No stereo panning, reverb, EQ, or editor preview spatialization. Existing audio playback and preview kept. Existing level audio lifecycle limitations unchanged.

## HUD
- Optional `Widget.group` string. Equal non-empty names create a window container in standalone player. First widget anchor supplies initial position; group lays children vertically. Blank group preserves existing adaptive individual buttons. Grouped actions support existing left/right/jump/label. Designer still uses flat preview; no nested hierarchy, inventory data model, or full menu editor.

## GPU lab
- Optional `gpu-lab` feature and `GPU_LIGHTING_LAB.bat`. Actual Bevy Material2d binding code with generated linear normal map, animated point light, and analytic shadow from a fixed circular occluder. Isolated example, not scene lighting; no integration with scene objects or particle dispatch. Shader and Rust code have not been compiled here.
- API reference: Bevy v0.16.1 `examples/shader/shader_material_2d.rs` and `assets/shaders/custom_material_2d.wgsl` at https://github.com/bevyengine/bevy/tree/v0.16.1 .
- Existing experimental particle compute shader and CPU fallback unchanged.

## QA
- QA matrix adds opt-in lab compilation. Run `QA_MATRIX.bat`, then `BUILD_WINDOWS.bat`. Run `GPU_LIGHTING_LAB.bat` separately to inspect lighting. No Rust compiler or Windows GUI here. No claim of FPS, EXE size, GPU or audio playback validation. All new Rust code remains unverified.
