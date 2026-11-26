# Forge Engine v0.7.23 — Inventory PRO & Checkpoints

Source update based on v0.7.22. No dependency additions or automatic rig/animation generation.

## Implemented in source (Rust compilation pending)
- Per-project item catalog in Gameplay PRO. Item names match stacks and pickups exactly. Five equipment categories: weapon, armor, accessory, head, offhand. Equip/unequip moves a single item atomically; replacing gear requires room for old gear. Damage bonus applies to generic Combat strikes (not Signal Lost's special demo attack). Heal restores living generic Combat actor health; optional named numeric variable delta. No consumable is spent if there is no applicable effect. Old items without definitions are still collectible. Existing inventory JSON has serde defaults for equipment.
- Versioned checkpoint (F5/F9 or pause-menu buttons), per entry scene `*.checkpoint.json`, max 32 MiB, separate from authored scene. Includes materialized project, active campaign level, inventory, demo health/collected/win state, elapsed clock and progress. Writes to unique temp file and renames after sync; load validates before replacing state. Physics velocity/contact state, Lua VM, active audio playback and GPU buffers are reconstructed, not exact snapshots. Local checkpoint files are trusted like project files; do not use untrusted saves. Source level must remain available and match identity. Not a cross-version migration framework.
- Visual animation event track: click to seek, drag event marker, existing event list remains. New non-positive variable transition (missing variable counts as zero).
- Atlas packer tries natural and descending-height shelf orders, selects smaller successful result while preserving frame index order. Optional single-frame, no-physics trim with object size and pivot compensation; multi-frame compensation requires per-frame geometry and remains pending. Original texture not modified.
- Double-click recursive docking divider to balance. Existing workspace and vector icons retained; not a complete visual redesign.
- Refresh all instances of selected prefab atomically, including newly added nested descendants. External nested-prefab references, deletion/reparent merge and automatic live synchronization remain pending.
- GPU lighting picks top 1–3 lights and top four blockers using bounded stable selection instead of sorting every candidate. Still scans all lights/solids. WGSL unchanged; no measured FPS benefit yet.
- Player refreshes sprite texture handles if a restored object has a changed asset. Build label/Windows scripts updated. Lock migration only changes local package version; patch excludes Cargo.lock and target.

## Not implemented
True hardware instancing, real-time EQ/Reverb/DSP, full editor redesign, external nested prefab references, generalized animated per-frame trim geometry. Existing indexed particle renderer and offline PCM16 WAV effects remain unchanged.

## Windows verification
1. Back up v0.7.22 and extract update ZIP at source root; keep Cargo.lock and target.
2. Run QUICK_CHECK.bat, QUICK_TEST.bat, QUICK_BUILD.bat. Send logs if any fail.
3. RUN_INVENTORY_PRO_SCENE.bat: I opens inventory; select Medkit and Use (health starts 1/5); select Training sword and Equip then Unequip; Energy cell Use increases energy HUD; F5 save, change inventory/position, F9 load; verify old scene unchanged. E collects nearby pickup.
4. Open `examples/inventory_pro/project.forge.json` in editor and edit Gameplay PRO item catalog. Test event marker dragging and single-frame no-physics sprite trim. Test multiple linked instances before and after new child added.
5. RUN_GPU_SCENE.bat and RUN_GPU_QUADS_SCENE.bat for graphics regression. CHECK_WINDOWS.bat before release.

## QA status
17 Python static/fixture tests pass; 4 campaign scenes (214 objects) pass validation. Rust compilation and Rust tests not run: no Rust toolchain in authoring environment. No runtime benchmark or new verified Windows build. GPU hardware instancing and DSP not claimed.
