# Forge Engine v0.7.31 — Timeline, Asset Cache & Render Diagnostics

Source candidate based on exact v0.7.30 source archive. No Auto Rig or automatic animator. No new Cargo dependencies.

## Changes
- Animation: explicit Rectangle selection mode on property graphs, drag selected group horizontally in normal mode, preserve relative key spacing, reject overlaps and out-of-range group edits transactionally. Four curve presets; existing Y-only Bezier handles retained. Does not implement independently adjustable Bezier X coordinates or multi-channel group drag.
- Asset import: persistent FNV-1a content-hash cache `.forge-import-cache-v1.json` (16,384 entries, 4 MiB), invalidated by size and modification timestamp. Reuse still requires exact byte equality. Cache is advisory; same-size edits preserving modification time may miss duplicate candidates; delete only the cache file to force reindex. No asset deleted. Existing folder scan and import limits retained.
- Asset database: scan PNG/JPEG/WAV/OGG under assets; missing stale paths skipped; reverse dependency list grouped by scene/prefab object, potentially unused filter and up to 256 parse warnings. Not a full transitive dependency graph: scripts, dynamic paths, unparsed files and assets outside recognized paths may be missed. Never use this list as an automatic delete list.
- Prefab graph: recursive canvas of reference nodes with parent edges and selected-node navigation into a composition. Existing direct-reference drag and layout auto save retained. Descendant canvas is read-only except selection; edit a nested composition by opening it separately. No full editable recursive graph yet.
- Design: clearer headers, project asset heading and selected asset text, inspector object name/dimensions/layer, viewport resolution display. Not a complete redesign.
- Performance: opt-in Bevy 0.16.1 RenderDiagnosticsPlugin (`FORGE_GPU_PROFILE=1`), GPU elapsed timings per supported recorded pass, explicit compute particle span, editor profiler display and JSON export `gpu_pass_ms`; Player logs diagnostics when enabled. Empty map if no samples; latest measurements may be delayed/stale. Do not sum nested spans. Total GPU frame and total VRAM remain null. Vulkan/DX12 timestamp support depends on GPU/device capabilities. Sampled process CPU usage and peak RSS added; peak is only sampled while profiler is open. No measured performance gain claimed.
- Build: root package version migration covers 0.7.14–0.7.30 without changing dependency pins; updated Windows package and log version. No target or Cargo.lock in ZIPs. Existing examples/shaders retained.

## Windows acceptance (not executed)
1. Back up project, extract patch over v0.7.30. Run QUICK_CHECK.bat and QUICK_TEST.bat, then QUICK_BUILD.bat. Preserve target and Cargo.lock. Report log on any error.
2. Open examples/visual_channels/project.forge.json, select independent channels, use Rectangle selection, then turn off mode and drag selected group. Verify spacing, bounds, collision rejection, copy/paste, save/reopen, undo. Check curve presets.
3. Import PNG batch twice, inspect cache file, change file size and repeat; refresh database and inspect reverse dependency list and unused candidates. Check malformed scene warning.
4. Open composite in examples/frame_geometry, inspect recursive canvas and select external node; open nested composition only after saving/discarding current draft. Confirm existing direct layout save/reload.
5. Run RUN_EDITOR_GPU_PROFILE.bat after successful QUICK_BUILD; open Assets & performance, inspect GPU timestamp diagnostics and export JSON; enable particles in GPU example for compute span. Confirm normal editor launch still works without profiling. No GPU timing should be inferred when no samples.
6. Run CHECK_EXPERIMENTS.bat separately if needed. GPU instancing runtime is not integrated.

## API reference
Bevy v0.16.1 source: https://github.com/bevyengine/bevy/blob/v0.16.1/crates/bevy_render/src/diagnostic/mod.rs and https://github.com/bevyengine/bevy/blob/v0.16.1/crates/bevy_diagnostic/src/diagnostic.rs . GPU timing path suffix `/elapsed_gpu` is from render diagnostic internal implementation at the same tag.

## QA limitation
Cargo/rustc not present in authoring environment. Static Python checks are not a Rust build or GPU execution. Added Rust group-move and persistent-cache tests, not executed here. Existing source is not certified as compile-ready. No Windows EXE provided.
