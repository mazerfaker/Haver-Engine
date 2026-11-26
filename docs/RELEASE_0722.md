# Forge Engine v0.7.22 — Inventory, Animation Events & Prefab Growth

Base: v0.7.21; no dependency changes, no Auto Rig or automatic animation authoring.

## Implemented in source
- Inventory Engine: opt-in per-project inventory; 1–64 slots, 1–99 units per stack, atomic capacity checks, merge/swap via click or drag. Player I opens inventory; E collects nearest leaf pickup within 80 world pixels. Explicit save/load (`inventory-save.json` in project folder); pickups are tracked by UUID and restored from the original scene on load when needed. Author initial slots in Gameplay PRO > Inventory Engine; set pickup in Inspector > Inventory pickup. Save is inventory-only, not full game state. Inventory is shared for the current campaign folder; different projects should use different folders. No equipment, item use effects or shop system.
- Animation: per-clip named time markers, crossed-time and loop handling, event log and optional Lua `on_animation_event(name)` callback. Transform key creation/update at playhead, key editing/sort, graph context menu to remove incoming/outgoing links. Existing list edits transition conditions.
- Sprite Studio: trim transparent margins of selected irregular atlas region on demand, with bounded image decoding; original texture unchanged. Fully transparent regions remain unchanged. Store clip to persist. Frame dimensions still scale to object size; trimming does not preserve visual size or pivot automatically.
- Linked Prefabs: explicit Refresh + new children adds new source members with UUID remapping and parent-relative translation, preserving local overrides and existing transforms. Deletions and changed parents still rejected atomically. Does not recursively resolve external linked prefab files or perform auto-sync.
- Workspace: right-click tab to join/split relative to other panels; previous drag docking and workspace serialization unchanged. Dark palette refined. Not a full redesign.
- GPU particles: fast mesh path uses four vertices and six U16 indices per particle instead of six duplicated vertices. Existing compute and draw shaders unchanged. Geometry payload (position + UV only) goes from 120 to 92 bytes/particle, excluding GPU alignment or allocator overhead. No hardware instancing, no measured speedup.
- QA: quick check now also checks minimal Player; lock migration handles 0.7.21 and older supported versions. New inventory fixture and RUN_INVENTORY_SCENE.bat.

## Verification
- 4 Signal Lost levels / 214 objects static validation.
- 14 Python static tests passed.
- 47 existing example files and 9 existing WGSL files byte-identical to v0.7.21 archive.
- JSON and TOML parsed; Cargo manifest differs only in root version.
- Nine new Rust tests added, not executed. Rust toolchain is unavailable in this environment; no Windows EXE, shader validation or runtime benchmark was performed. v0.7.15 remains last confirmed full Release.

## Windows update
Back up your project. Extract update into v0.7.21 root, replace files, keep Cargo.lock and target. Run QUICK_CHECK.bat, QUICK_TEST.bat, QUICK_BUILD.bat, RUN_EDITOR_DEBUG.bat, RUN_INVENTORY_SCENE.bat, RUN_ATLAS_SCENE.bat and RUN_GPU_QUADS_SCENE.bat. Do not treat static checks as a confirmed build.
