# Forge Engine v0.7.33 — Multi-track Timeline & Bus DSP

Source candidate based on exact v0.7.32 source archive. No Cargo dependency changes, Auto Rig, or automatic animator.

## Animation
- New Multi-track timeline under independent property channels: six lanes, rectangle time/lane selection, switch to Move selection, drag preview, commit on release. Uses existing atomic multi-channel move; conflicts reject whole operation. Click seeks playhead. No multi-clip or event-track group edits.
- Scalar Bezier Y handles now allow -2..3, X stays 0..1; Overshoot preset. Legacy curves remain compatible. Scale/opacity output still clamped to supported limits. Global Key transform curves remain unchanged.
- Fixed property-only transform tracks in simulation by providing base object position/rotation when no legacy or clip-wide keys exist. Player/physical body restrictions unchanged.

## Audio
- Custom buses now optionally carry WavEffects and built-in DSP presets, saved in project JSON. The nearest enabled ancestor preset is inherited; local EQ gains add (bounded), room/reverb combine with bounded wet amounts. Each source is processed independently; this is NOT shared post-sum bus DSP. Existing PCM16 WAV restrictions, max 16 DSP sources and 32 MiB PCM budget remain.
- F8 mixer edits previously configured bus DSP live. Enable a preset before scene start so streaming decoder is allocated. Editor preview still auditions local effects only, avoiding accidentally double-applying inherited effects.

## Assets / Prefabs
- Dependency browser has selectable direct document edges, previous-node back action, active-scene shortcut, missing referenced asset reporting and explicit uncached rescan. Obsolete absent and unreferenced database entries pruned; no asset files deleted. Cache identity/size/mtime strategy otherwise unchanged. Dynamic Lua references are not resolved.
- Composite editor supports renaming and reparenting up to 128 displayed local objects; does not alter world positions. Changes saved using existing conflict check, full graph validation and backup. Nested external references still edited by opening their composition, not by editing arbitrary nested objects in one scene. Linked instance topology changes remain guarded.

## Editor / Performance
- Refined shared window margin and slider/indent spacing. No comprehensive redesign.
- Fixed-size stack candidate ranking for up to three lights and four blockers, preserving stable tie ordering. Audio mixing early-out if no audio entities. Avoid cloning composite object document until actual UI edit. Existing GPU materials and shaders unchanged. No measured FPS/VRAM improvement; GPU instancing stays isolated.

## Manual checks
Run QUICK_CHECK.bat, QUICK_TEST.bat, then CHECK_EXPERIMENTS.bat and QUICK_BUILD.bat. RUN_TIMELINE_OVERSHOOT_SCENE.bat shows a property-only clip with Overshoot. Open examples/timeline_overshoot/project.forge.json in editor, select Property-only overshoot, Animation Studio → Independent property channels → Multi-track timeline. Select a rectangle, switch to Move selection, drag then release. Test conflict rejection. RUN_BUS_DSP_SCENE.bat compares local dry and inherited DSP sources. Press F8 and adjust Wet room preset. In Assets & performance, rescan without cache and follow direct document links. In a composite, rename/reparent local objects; attempt cycle and confirm save rejection.

## Limits
Cargo/rustc unavailable here; no Rust compilation, Rust test execution, Windows GUI, audio, GPU rendering or benchmarks confirmed. Five new Rust tests authored. Static regression suite and ZIP reconstruction are not a substitute for compile/runtime checks. Existing unresolved compile regressions cannot be certified fixed. Patch excludes Cargo.lock and target. prepare_lock.ps1 migrates only the root package version.
