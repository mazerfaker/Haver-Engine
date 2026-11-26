# Forge 0.7.8 — modular runtime and cache

Default editor enables all existing features. `BUILD_PLAYER_ONLY.bat` preserves full player features. `BUILD_MINIMAL_PLAYER.bat` removes Rapier, Lua, audio and CPU lighting; projects requiring them are rejected at startup or level transition. Bevy 2D and egui remain required. Feature combinations are not compiled here.

Workspace menu: viewport cache 32–1024 MiB, saved with workspace settings. Profiler reports retained decoded texture bytes, hits, misses and evictions. LRU is used when shrinking the budget; current-scene admission remains bounded and unused paths are removed. This is not total RAM/VRAM; queued decodes and other caches are excluded. Viewport requests textures only for visible sprites unless game preview is open. Existing Bevy render culling is retained. Chunk batching is not implemented.

GPU shaders from 0.7.7 remain prototypes, not connected to Bevy. Dynamic shadows and GPU compute dispatch remain outstanding. No Auto Rig.

Run CHECK_WINDOWS.bat and BUILD_WINDOWS.bat. Compare benchmark.json from both versions with `python tools/compare_benchmarks.py old.json new.json` on the same hardware/profile/project. No performance gain is claimed without measurement. Rust/Windows compiler unavailable in authoring environment. Static demo validation only.
