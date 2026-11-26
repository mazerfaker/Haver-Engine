# Forge Engine v0.7.41 — Persistent Timeline & Async Transactions

Source candidate based on the exact v0.7.40 source archive. No dependency changes; no Auto Rig or automatic animator.

## Installation
Extract the update ZIP over v0.7.40 (backup project first). Keep `Cargo.lock` and `target`; `prepare_lock.ps1` migrates only the root forge_engine package version. Run `QUICK_CHECK.bat` and `QUICK_TEST.bat`. Full source archive is also provided. No executable is supplied.

## Stability & QA
- Fixed missing explicit `#[path]` for the editor's animation_studio module (crate root is src/bin/forge_editor.rs).
- Fixed Windows CI artifact name that still referred to v0.7.29.
- Cargo check attempted but blocked: cargo is not installed in this environment. Rust tests, Windows compilation, actual editor UI, audio and GPU rendering remain unverified.
- 95 Python validation tests + 2 packaging tests passed. Includes CLI transfer/rollback with approval fingerprint, stale-plan refusal, duplicate journal path refusal and symlink journal refusal. Earlier fixture checks preserved.

## Animation Studio Ultimate
- Timeline zoom, horizontal scroll and event lane inclusion persist in editor-only `.timeline.json` sidecars per saved scene/object/clip. Curve axis, displayed duration and value range persist in `.curve.json`. Existing Preview preferences remain unchanged.
- Apply Linear, Ease In-Out XY or Overshoot XY to selected keys in one channel, alongside previous group interpolation tools. Existing independent Bezier channels, group clipboard and event timeline retained. No new interpolation format.
- Settings saved after pointer release; explicit Save timeline preferences button available. Playhead, group selection, and playback state not persisted. Sidecar writes bounded and atomically replaced.

## Asset Database / Prefab Graph Ultimate
- Existing Transactional Move / Rename now starts a background worker for dry run; editor displays a separate confirmation window with plan fingerprint and then starts an asynchronous apply. Normal editing/saving shortcuts paused while job is active. Reload scene and clear caches after completion. No forced cancellation during a write. Closing the process during commit is discouraged; journal retained for recovery.
- `File transactions → Recover file transaction (journal)` opens folder picker and confirmation; rollback in worker via existing journal CLI. Requires a saved scene that can be opened, a single tab and empty undo/redo history; for unreadable scenes use the existing command line recovery (`py -3 tools/relocate_asset.py PROJECT_ROOT --rollback JOURNAL_FOLDER`) with all editors closed.
- Prefab library → Transactional subtree transfer: source/destination simple library filenames, subtree UUID and optional destination parent UUID. Dry run and apply require matching SHA256 fingerprint. Prefab transfer only works for unused, unscripted prefabs under prior safety policy. Save composer edits before starting; reopen affected composer after commit. No linked-instance migration or drag-and-drop between files.
- Recovery rejects aliased journal folders, duplicate paths, invalid states and changed paths during rollback. Existing CLI tools and safe copy/relink retained. Not a guarantee of atomicity across arbitrary external processes or power loss. Close other editors/players; dynamic Lua references still block unsafe transactions.
- Python 3.10+ required. Native file chooser remains synchronous, disk scanning/apply/rollback runs on worker.

## Audio Mixer Ultimate
- F8 shared-bus meter displays pre/post sample peak, limiter reduction, and pre/post RMS over latest approximately 50 ms audio window. Stereo channels combined for RMS. Values are measured before external sink/master gain; not true peak or LUFS. Last snapshot can remain displayed when playback pauses. Existing stereo-linked limiter ceiling 0.98 and release 80 ms retained; no new limiter algorithm.
- Meter accumulates locally in decoder; atomic publishing at window boundaries or end of stream, reducing atomic operations relative to per-frame publication. New Rust test for RMS/peak window calculation added, not executed. No measured CPU savings.
- Previous Float32, windowed sinc resampling and experimental cascade retained.

## Lighting / Performance Ultimate
- Resident Normal Map cache checks file length and modification time at 500 ms interval (maximum 8 maps); changed/deleted maps are invalidated, failed loads retried on metadata change. Same-size same-mtime edits may require reload. Existing render-world texture ownership preserved.
- Fully transparent sprites skip light candidate iteration; hard shadows use direct single ray without perpendicular/jitter calculation. Existing five-ray soft shadow algorithm preserved. No new shader quality profile.
- Existing GPU timestamps, CPU/RAM stats and RGBA8 payload estimate preserved. Total VRAM and measured FPS/RAM improvement remain unavailable.
