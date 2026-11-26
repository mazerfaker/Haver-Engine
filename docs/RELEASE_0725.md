# Forge Engine v0.7.25 — Animation Channels & Experimental Runtime Labs

Source update based on v0.7.24. Not a compiled Windows build. No Auto Rig or automatic animation authoring.

## Changes
- Animation: optional independent transform tracks per named clip (`motion.clip_keys`), with clip-local timing and wrap at the sprite clip duration. An empty explicit track disables the global motion track. Legacy scenes continue sampling the global track in scene time. Enable `Independent clip track` in Animation Studio; `Copy global keys` seeds the independent track. Supports position/rotation and existing interpolation modes. Runtime transform motion retains the previous restriction to non-player objects without physics bodies. The legacy inspector curves remain global. Instance creation offsets both global and clip-local keys; linked refresh retains local transform tracks as before.
- Atlas: try six shelf widths (256–2048 px) and three frame orderings, retain the smallest successful area, restore original frame sequence. New `.atlas.json` sidecar records pixel rectangles and source filenames at import time, not subsequent manual edits. Previous frame geometry and trimming remain unchanged. No external packer or dependencies.
- GPU lab: dedicated `gpu-instancing-lab` feature. A compute pass updates 1024 vec4 particle slots (16 KiB payload), then a separate 2D render pass invokes `draw(0..6, 0..1024)` and obtains particle state by `instance_index`. No CPU per-particle vertices, vertex buffer or readback. Existing shader drives simulation with a frame-dependent step. This standalone post-process pass replaces the lab camera output, not the existing runtime renderer. Not yet compiled or visually tested. Fixed aspect particle shape; no editor emitter integration yet.
- Lighting: a bounded grid is built for 16–4096 active lights; 256-unit cells, large lights retained in a global list, ordered merge preserves original tie ordering, small or oversized scenes use the prior full scan. Shadow algorithm and previous shaders unchanged. Grid rebuild cost and speedup not benchmarked.
- Audio: `StreamingEffects` maintains crossover filter and ring buffer state across arbitrary mono/stereo blocks (8–96 kHz), two-band gains and early reflections with a bounded feedback tail. No allocation in the processing loop; separate laboratory source uses the Bevy decoder callback, plays synthetic pulses and changes preset from atomics every 256 samples. Not wired to existing scene AudioPlayer, no production mixer reverb. Parameter changes are not smoothed in this experimental lab.
- Prefabs: `Visual Prefab Composer` in library allows selecting external prefab files, adding multiple references, setting XY offsets and optionally attaching all references to selected subtree. Saves a new composite document only, refuses existing filenames and validates expansion before accepting; existing nested reference limits and refresh behavior remain. No drag-and-drop canvas or editing existing composites.
- Editor: dark theme color and state contrast refinement. Existing docking layout and icons retained. Not a comprehensive redesign.
- Build: dependency declarations/profiles unchanged, two isolated optional feature flags for labs. `prepare_lock.ps1` migrates root package from v0.7.14–24 without touching other resolved versions. Update ZIP omits Cargo.lock and target.

## Windows validation order
1. Back up the current project directory. Extract update from v0.7.24 into that folder (not a clean copy) and retain Cargo.lock / target.
2. `QUICK_CHECK.bat`, `QUICK_TEST.bat`, `QUICK_BUILD.bat`, `RUN_EDITOR_DEBUG.bat`.
3. `RUN_ANIMATION_CHANNELS_SCENE.bat`: free non-physics object moves horizontally once, then loops vertically; the player remains independently controlled. Inspect the clip tracks in Animation Studio.
4. `RUN_FRAME_GEOMETRY_SCENE.bat`, `RUN_GPU_SCENE.bat`, `RUN_GPU_QUADS_SCENE.bat` for regression checks.
5. `CHECK_EXPERIMENTS.bat` separately checks new experimental examples. `GPU_INSTANCING_LAB.bat` should render animated orange particles; `AUDIO_DSP_LAB.bat` plays short synthetic pulses; 1=dry, 2=low boost, 3=high boost, 4=room feedback, Escape=exit. This is a test tone, not music.
6. Test Visual Prefab Composer in a saved project with library files; verify source files are unchanged and new composite can be instantiated. Then consider Release build.

## QA scope
24 Python tests (22 validate + 2 packaging), four Signal Lost fixture checks, JSON/TOML parse, archive integrity and exact patch reconstruction. Rust tests added for clip-local sampling, grid coverage, safe composite creation and streaming continuity, but not executed. Rust compiler absent. No performance measurements, Windows binary, WGSL execution or audio playback verified here.

API references used for the experimental paths: Bevy v0.16.1 custom_post_processing.rs and decodable.rs (official Bevy repository). Independent implementation; no external dependencies added.
