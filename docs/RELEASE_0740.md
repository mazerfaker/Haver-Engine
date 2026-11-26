# Forge Engine v0.7.40 — Persistent Preview, HQ Audio & Safe Transactions

Source candidate based on the exact v0.7.39 source archive. No new Cargo dependencies; no Auto Rig or automatic animator. Windows compilation remains unverified because Cargo and rustc are not installed in the working environment.

## Changes
- Animation Studio: fixed an inherited undefined `id` in `atlas_canvas`. Per-scene/object/clip Preview sidecar `.forge-preview-<UUID>-<hash>.json` persists zoom, pan, Onion Skin, RGBA colors, ghost counts, fade, and height. Saves changed settings after mouse release; manual Save preview preferences is also available. Reads bounded 16 KiB, clamps fields, rejects symlinks, writes via temporary file. Timeline zoom up to 32×, Center playhead, Zoom selection, numeric Bezier X and Y. Scrubbing in Animation Studio stops autoplay. Existing animation file schema stays intact; Timeline zoom itself is session-only.
- Asset Browser: `Transactional Move / Rename` runs the existing Python journal tool directly, previews plan and asks confirmation, checks SHA256 plan fingerprint on apply, reloads saved scene and invalidates editor caches/history after successful commit. Requires Python 3.10+, saved scene with no undo/redo, one document tab, stopped play, and no other editor/player modifying project. The GUI intentionally refuses dynamic/embedded Lua references instead of silently changing them. The offline CLI still has its explicit `--allow-dynamic` option. Per-object atlas-state metadata now updates along with `.atlas.json` sidecars. Copy/relink remains available. The portable packaging script includes both Python tools. GUI is synchronous, so a large scan can temporarily block UI; no independent interprocess lock or full crash-consistency guarantee across arbitrary filesystems. If commit succeeds but reload fails, close without saving and recover from journal.
- Audio: bounded 32-tap Blackman-windowed sinc filter, 257 fractional phases, low-pass cutoff for rate conversion, normalized DC gain; tiny streams (<16 frames) retain linear interpolation. Float32 buffers, mono/stereo conversion, 8–96 kHz and prior budgets retained. Shared/cascade decoder processes a full interleaved frame with stereo-linked sample-peak limiter (0.98 ceiling, instantaneous attenuation, 80 ms gain recovery). Child DSP stages no longer hard-clip at ±1; bounded ±64 for safety. Legacy single-source path and offline exporter preserve their existing behavior. F8 → Shared bus peaks / limiter shows pre/post peaks and gain reduction accumulated since previous read, before external sink/master gain. Not true peak, not LUFS, not lookahead. New summed/cascaded mode remains opt-in. Native audio/device behavior not tested.
- Lighting: bypass normal texture fetch when using flat RGBA8 fallback, skip shadow rays when diffuse contribution is zero. Existing soft-shadow sample count, material format, and light selection preserved. One WGSL shader changed. Track normal map count and estimated RGBA8 texel payload in profiler and JSON export. Not total VRAM or driver allocation. No measured FPS or memory improvement.
- Prefab: `tools/transfer_prefab.py` transfers a local subtree between two saved, unused library prefabs. Keeps UUIDs, internal parents and world transforms, preserves unknown JSON fields, optionally attaches to a destination parent. Uses shared journal/rollback. Refuses source/destination consumers, scripts, moved external attachments, linked objects, remaining cross-file UUID references, collisions and empty source. Does not migrate existing linked instances, graph layout sidecars, or Lua references. CLI only; editor shows usage hint. Existing Prefab Composer retained.
- Build: startup status reads Cargo package version; root lock migration accepts 0.7.39 without changing dependency pins. Existing Windows check/test scripts retained.

## Windows acceptance
1. Back up the local project. Extract the update ZIP over the existing v0.7.39 source directory. Keep Cargo.lock and target. Run QUICK_CHECK.bat (the lock helper migrates only the root package version), then QUICK_TEST.bat. If either fails, send forge-quick-check.log / forge-quick-test.log. Do not treat this as a confirmed stable release yet.
2. Animation Studio: open a saved scene, change Onion Skin/zoom/pan, release mouse, reopen editor and verify same clip settings. Check irregular atlas editing, scrub playback, Center playhead and Zoom selection.
3. Resource transaction: use a disposable project, select an asset in Asset Browser, enter a new existing-folder `assets/...` path, click Transactional Move / Rename. Confirm the dry-run plan. Verify scene reload, prefabs and atlas-state files; confirm old path removed and recovery journal retained. Python 3.10+ required. Close other Forge processes.
4. Audio: run RUN_RESAMPLED_BUS_SCENE.bat; F8 → Shared bus peaks / limiter. Test stereo balance, source/bus gain, EQ/Reverb, limiter, fallback, restart. No GPU or audio hardware tests were run here.
5. Profiler: RUN_EDITOR_GPU_PROFILE.bat; compare GPU pass times, normal-map payload and RAM on the same scene/settings. Actual total VRAM is unavailable.

## Cross-prefab CLI (disposable project first)
Close editors/players. Dry run:
```powershell
py -3 tools\transfer_prefab.py "D:\MyGame" a.prefab.json b.prefab.json <object-uuid> --parent <destination-parent-uuid>
```
Omit `--parent` to attach at root. Repeat with `--apply` to commit. Journal path is printed. Recovery for either transaction type:
```powershell
py -3 tools\relocate_asset.py "D:\MyGame" --rollback "D:\MyGame\.forge-relocation-<id>"
```
Rollback refuses conflicting edits. Preserve the journal until the project has been checked.

## Validation
88 Python validation tests and 2 packing tests executed successfully, including real file transaction commit/rollback, injected failure, approval mismatch, external consumer rejection, per-object atlas metadata, and independent numeric filter reference. New Rust tests are unexecuted. The Windows runtime/GUI, audio, shader and hardware performance remain unverified.
