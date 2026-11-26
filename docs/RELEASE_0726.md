# Forge Engine v0.7.26 — Studio Refinement & Optional DSP

Base: exact v0.7.25 source archive. No Auto Rig or automatic animator added. No dependency or profile changes; only Cargo package version changes. Existing demos, GPU instancing lab and indexed particle runtime retained.

## Changes
- Player: optional per-source streaming EQ and short room-reflection/feedback effect for mono/stereo PCM16 WAV (8–96 kHz). Settings in Inspector > Audio > Streaming DSP, applied on next scene start/restart. A custom Bevy audio asset runs the existing StreamingEffects processor per sample. Normal playback fallback for unsupported input, invalid paths or budget. Max 16 DSP sources, 8 MiB input per source, 32 MiB PCM payload per scene; audio file is loaded into memory before playback, not streamed from disk. Non-looping wet sounds have a 0.5-second tail. Mixer/spatial controls continue through the same AudioBus. No live parameter updates, full reverb, or editor audio preview DSP yet.
- Animation: automatic loading on changing stored clip in dropdown, stores outgoing known clip; duplicate key at +0.1 seconds, three separately normalized curve previews sampled from the current transform track. Existing independent clip tracks remain. No new automatic animator.
- Sprite: manual normalized frame geometry editor and reset; Store clip + sync atlas metadata writes `assets/atlas-state-<object UUID>.json` (schema 2) with all clips, pixel rectangles, geometry, pivot and logical size. Original import sidecar and image remain untouched. Explicit action, not automatic per drag. Metadata capped at 16,384 total frame entries and 8 MiB serialized. Shared atlas objects do not overwrite each other's metadata. Export metadata is not a new runtime import format.
- Prefabs: Open composition, change external refs and offsets, choose local parent, save with uniquely named `.bak` backup. Validates proposed full dependency graph in memory, detects external changes against original bytes, retains reference identities; replacement via temporary sibling file. Existing materialized instances refresh separately using previous safety rules. List-based hierarchy, not a drag-and-drop node canvas.
- Lighting: shader uses proper unbounded slab interval on parallel axes, five evenly spaced soft-shadow rays rather than three, early exit on first blocker. Hard shadows use one ray. Soft shadows can cost more GPU time; actual output and performance need checking.
- Editor: larger interaction targets and docking drop tooltips; recursive/classic docking retained. Not a complete redesign.
- GPU hardware instancing: unchanged lab only. The user requested integration after validation, which has not yet been supplied. Existing indexed particle runtime remains default.
- QA: 4 additional Python static tests, 4 new Rust tests (not executed); CI optional experiment compile step (not run). Existing scenes and shader labs preserved.

## Run on Windows
1. Back up v0.7.25. Extract update archive in project folder. Do not delete Cargo.lock or target. The prepare_lock script migrates only the local package version from 0.7.14–0.7.25 to 0.7.26.
2. QUICK_CHECK.bat, QUICK_TEST.bat. Send forge-quick-check.log / forge-quick-test.log on failure. QUICK_BUILD.bat and RUN_EDITOR_DEBUG.bat only after check passes.
3. RUN_AUDIO_RUNTIME_SCENE.bat: move left to dry pulse (green), right to filtered/room pulse (orange). Both use the same generated PCM16 WAV. R restarts scene. Inspect Audio component options in the editor. Audio device required. Settings are fixed during playback.
4. RUN_GPU_SCENE.bat: compare hard/soft shadows, vertical/horizontal blockers. RUN_ANIMATION_CHANNELS_SCENE.bat and RUN_FRAME_GEOMETRY_SCENE.bat: inspect animation and sprite placement.
5. CHECK_EXPERIMENTS.bat, GPU_INSTANCING_LAB.bat, AUDIO_DSP_LAB.bat. Lab results not confirmed. Only then BUILD_WINDOWS.bat for release.

## Validation limits
Rust toolchain absent in the working environment. No Rust compilation, actual shader validation, executable, runtime audio or FPS/RAM/VRAM measurement. Python tests are static fixture/source checks, not proof of Rust behavior. v0.7.26 is a development source snapshot, not a stable release.
