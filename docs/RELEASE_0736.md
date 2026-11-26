# Forge Engine v0.7.36 — Synchronized Preview & Live Audio Gains

Source candidate based on the exact v0.7.35 full archive. No new Cargo dependencies, no Auto Rig or automatic animator.

## Changes
- Stability: repair out-of-scope `e` in editor brand function; move RSS/CPU details to profiler; version heading follows Cargo package version.
- Animation: read-only synchronized playhead panel samples the current clip's frame, position, rotation, scale, opacity and adjacent event markers. The shape preview is schematic, not the rendered sprite. No Lua event dispatch during scrubbing. Existing animation tools retained.
- Asset database: bound queued transitive nodes to 32768, avoid duplicate enqueue, reset in-memory database and graph caches on project path change, search resource list by name, restore active-scene navigation when owner is absent. Existing on-disk caches retained. Same-path scene edits require refresh. No asset deletion.
- Experimental summed audio: per-track atomic gain controls connected to existing F8 source sliders; smooth over ~20 ms per sample frame (stereo aligned); removed sources fade to zero. Compose all ancestor DSP parameters for summed buses (EQ dB sum clamped ±12, room/reverb combined wet proportions). Existing per-source nearest-preset semantics unchanged. NOT full cascaded parent-bus signal routing or arbitrary format/spatial mixing; prior fallback retained. Runtime experimental, off by default.
- Prefab graph: recursive reference identity scoped by parent UUID, save positions on drag release and reload in a separate `.recursive-layout.json` sidecar. Old direct-node layout remains intact. Existing nested hierarchy editing retained. No cross-file drag reparenting. Layout read errors fall back to defaults.
- GPU: conservative blocker bounds account for animated visual dimensions; remove unused owned normal-map image handles after updating/removing materials. No shader changes. More conservative queries can increase candidate counts. No FPS/RAM/VRAM improvements measured.

## Verification and installation
67 Python tests passed (65 validate + 2 pack tests); four Signal Lost levels with 214 objects passed static validation. Rust compilation attempted: `cargo: command not found`. Rust tests, Windows binary, live audio and GPU runtime not verified. No claim of stable build.
Extract the update ZIP over v0.7.35, retaining Cargo.lock and target. The existing prepare_lock.ps1 migrates only the root package version; dependency pins remain unchanged. Run QUICK_CHECK.bat then QUICK_TEST.bat and only if successful QUICK_BUILD.bat. RUN_LIVE_SUMMED_BUS_SCENE.bat tests F8 gain controls. Check the synchronized preview panel and recursive prefab layout after reopening the editor.
