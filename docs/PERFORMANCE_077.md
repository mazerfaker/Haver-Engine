# Forge Engine 0.7.7 — performance and scope

Base: exact v0.7.6 source archive. No Auto Rig or external animator dependency.

## Changes
- `editor` Cargo feature (default) gates rfd and sysinfo and the editor binary. `BUILD_PLAYER_ONLY.bat` builds only the player without these dependencies. Bevy, egui, Rapier and Lua remain required; this is partial modular compilation, not a minimal renderer-only build.
- Startup metric: main entry to first successful egui context, shown in MAX profiler. Not OS process startup or first presented GPU frame. Game preview texture starts at 1x1 and grows when active.
- Native viewport texture cache: 128 MiB admitted decoded RGBA payload, 8192 dimension limit, bounded job/result queues retained. Rejected textures use fallback; retries occur after stamp change or cache removal. This is not a total RAM/VRAM limit. Separate egui thumbnails, transient buffers, Bevy assets and render targets are outside the budget. Native mode no longer loads full egui duplicates unless animation tab needs them.
- Player sprite synchronization uses ID maps instead of repeated linear scans. Bevy built-in visibility behavior retained; chunk batching and explicit frustum culling not added.
- Spatial grid broad phase for horizontal legacy AABB no-overlap cases. Original ordered collision resolution retained when overlap is possible. Rapier player velocity is changed only when different, allowing existing automatic sleeping behavior. Vertical and trigger scans unchanged.
- HUD layout preview: drag widgets to edit normalized anchors. Existing widget serialization retained. Audio mixer unchanged.
- Experimental standalone WGSL lighting and particle sources are supplied, but not validated, dispatched, or used by the game. Lighting expects externally generated shadow mask; dynamic shadow generation is not implemented.
- Benchmark allocation is clamped before reserving memory. Existing three build profiles retained.

## Not completed
True docking, new editor icon set, full audio mixer, GPU render graph, dynamic shadow generation, GPU particle dispatch, chunk renderer, Signal Lost Deluxe combat/bosses/new levels. No new demo content; original scenes and assets retained. No claim of actual FPS gain or executable size reduction.

## QA
Run CHECK_WINDOWS.bat, BUILD_WINDOWS.bat and BUILD_PLAYER_ONLY.bat. Use BENCHMARK.bat on each version with the same hardware and build profile. Compare benchmark.json, build-sizes.json and MAX startup metric. Run GUI tests on large textures, native/fallback rendering, animation tab, HUD drag, and Rapier idle/movement. Rust compiler unavailable in authoring environment; compilation and runtime results are unverified.
