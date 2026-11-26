# Forge Engine v0.7.35 — Shared Audio & Timeline Events
Source candidate from the exact v0.7.34 archive. No new Cargo dependencies. No Auto Rig or automatic animator.

## Animation Studio
- Multi-track Timeline: Include event lane in copy/move, Events only, Delete events in selected range. A shared time shift validates key collisions, clip event duration and duplicate event names/times before committing any lane. Existing copy/paste and zoom retained. Group event marker highlighting added; property key drag preview retained (event markers update at commit).
- Correct frame timing for positive FPS below 1. Zero FPS still uses the old finite timeline fallback.
- Drain Lua emit/spawn queues after trigger and animation-event callbacks; previously queues could be cleared on the next step before consumption. Other Lua globals written by post-update callbacks are not newly synchronized.

## Audio Bus Runtime (experimental, explicit opt-in)
- Gameplay > Audio routing > Experimental live summed PCM bus. Per custom bus, direct scene sources with matching PCM16 mono/stereo format (8–96 kHz), speed exactly 1, same looping mode, no spatial/panning or source DSP. 2–16 sources; looping clips must also have equal lengths. Sources are aligned at scene start; shorter nonlooping tracks are padded with silence. One decoder sums samples before a single streaming EQ/reflections/reverb state. Bus gain/mute and inherited DSP are live via F8; source volume is latched at scene load. No sample-rate conversion, full hierarchical summed routing, spatial bus mixing, or independent source transports. Summation may clip at high levels; leave headroom.
- Incompatible groups fall back in full to the previous per-source path; opt-in is false by default for existing scenes. 32 MiB PCM budget, 8 MiB per input file, at most 16 DSP decoders across the scene. Loops use the decoder's own loop state, nonlooping shared tracks include a two-second tail. No new audio library.
- RUN_LIVE_SUMMED_BUS_SCENE.bat: two generated mono 16 kHz one-second synthetic tones, direct to Wet room. Verify console 'Live shared bus', both tones, F8 mute/gain and effect changes. Incompatible format or spatial flags should log fallback and use the previous path. Offline summed export retained.

## Asset Database & Prefabs
- Recursive asset browser with name and missing-only filters; cached bounded traversal (512 nodes, 16 levels), repeated nodes indicated. Refresh invalidates view cache. Fixed document traversal for prefab JSON paths under assets/ and missing entries. Existing canvas, direct/transitive lists and persisted cache retained. Dynamic script assets remain incomplete; no automatic deletion.
- Drag recursive prefab dependency nodes on their canvas (session-only layout, not transforms). Inline nested hierarchy supports drag onto another local object's handle to reparent; checks local cycles both before and during applying changes. File saving still uses original conflict check, dependency validation and backup. No cross-file drag/reparent or simultaneous multi-file commit.

## Lighting & Performance
- Lighting computes animation time once per sprite; shader returns transparent black for fully transparent texels before lighting/shadow calculations (both texture samples remain before the branch to preserve derivative uniformity). Existing shadow quality and normal-map behavior retained. No new normal-map cache or shadow algorithm.
- Reuse audio source index map between frames. Existing diagnostics and GPU timestamps retained. No measured FPS, GPU timings, CPU or VRAM gains.

## QA / install
- Python static/reference tests and static Signal Lost fixture checks are not Rust execution. New Rust unit tests are included but unrun. cargo check blocked: cargo not found. Actual Windows build, GPU, live sound and GUI not verified. Source candidate only, not a stable binary release.
- Update on v0.7.34 only. The ZIP omits Cargo.lock and target. prepare_lock.ps1 migrates the root package version, leaving dependencies pinned. Run QUICK_CHECK.bat, QUICK_TEST.bat, CHECK_EXPERIMENTS.bat, QUICK_BUILD.bat, RUN_LIVE_SUMMED_BUS_SCENE.bat and RUN_EDITOR_GPU_PROFILE.bat. Report errors with exact logs. GPU hardware instancing lab remains isolated.
