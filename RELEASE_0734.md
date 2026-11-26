# Forge Engine v0.7.34 — Timeline Clipboard & Summed DSP Tools
Source candidate based on the exact v0.7.33 archive. No new Cargo dependencies or auto-rig/automatic animator.

## Animation Studio
- Multi-track clipboard captures selected six property/visual lanes and optionally clip events, using one common earliest time. Paste at the playhead into the current named clip, preserving relative time, curves and event names. Transactional validation rejects duplicate keys/events, out-of-range events and key budgets; no partial change.
- Multi-lane canvas zoom 1–16×, scroll in seconds, Fit reset, event markers with hover names. Event editing still uses existing event timeline; group drag moves property keys, not events. Clipboard is session-only.

## Asset Database
- New clickable direct dependency canvas (up to 128 edges per selected document) with missing nodes and drill-down; prior recursive list and reverse dependencies retained.
- Cache the latest transitive traversal in editor memory until owner changes or database refreshed. Persistent parsed-document/hash caches unchanged. Dynamic Lua dependencies remain incomplete. No automatic deletion.

## Audio Bus
- SummedBus accepts up to 16 aligned PCM blocks with per-input gain, sums before one stateful StreamingEffects instance, without allocating during block processing. No connection to Bevy's live mixer. Existing inherited per-source runtime DSP unchanged.
- Gameplay > Audio routing > enable Inherited source DSP: Save/Load DSP preset JSON; Export summed WAV (offline). Pick 1–16 PCM16 WAV files, same rate and channel count, total <=32 MiB; tracks begin together, equal gain 1/N, shorter tracks padded with silence, optional two-second effect tail. Output uses selected bus EQ, reflections and reverb, not live positions, source settings or full bus routing. UI operation synchronous. Existing files unchanged unless user explicitly chooses an existing export path.

## Prefabs
- On Recursive dependencies canvas select a composite file, then Edit selected nested hierarchy here. Rename/reparent local objects and save the external file independently, without replacing parent draft. Original optimistic conflict checks, graph validation and backup retained. Changes affect every instance of the external file on future refresh. Legacy object-list prefabs cannot be edited here; no simultaneous multi-file transaction, arbitrary nested drag-reparent or in-place node editing.

## Lighting & Performance
- Reuse visible-entity set, normal-map usage set, and static-blocker vector capacity across sync frames. Remove obsolete material assets when owned visuals despawn. Expose owned lighting material count in profiler and JSON; not measured VRAM. Shader algorithm and prior GPU timestamp mode unchanged; no new measured FPS, RAM or VRAM data.
- All existing examples and shaders preserved. No new engine dependencies; update ZIP omits Cargo.lock and target. prepare_lock.ps1 migrates root package version only.

## Validation and limitations
- 58 Python tests passed (static/reference-model tests, not Rust execution), four Signal Lost levels 214 objects. Two new Rust tests included but unrun. `cargo check` blocked: cargo not found. Windows build, actual audio/GUI/GPU playback, runtime regression tests and resource-performance gains unverified. No live shared bus graph, complete recursive canvas editing or new shadow algorithm. GPU hardware instancing remains isolated.
- Apply update on v0.7.33 only; run QUICK_CHECK.bat, QUICK_TEST.bat, CHECK_EXPERIMENTS.bat and QUICK_BUILD.bat. Test multi-lane copy with events, paste collision rollback, zoom and scrolling; compare summed WAV with individual sources; select nested prefab and save/refresh. Use RUN_EDITOR_GPU_PROFILE.bat to inspect timing and material counts.
