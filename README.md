# Forge Engine v0.3.1 — Native Editor Refinement

Experimental source build. Not compiled or launched in the packaging environment.

## New editor tools
- Scene search; duplicate; focus selected; safe deletion with reparenting; parent cycle prevention.
- View menu: panel visibility, grid, collider outlines. Panels remain resizable, not freely dockable.
- Cursor-centered zoom, adjustable snap, screen-axis constraints, isometric grid.
- Q/E/R transform mode, Ctrl+D duplicate, F focus, Delete, Ctrl+S/Z/Y.
- Previous/next animation frame and editor camera markers.
- Existing v0.3 GPU viewport, fallback, runtime, Lua, graph and asset import remain in source.

## Build
Install Rust stable and Visual Studio C++ Build Tools, run BUILD_WINDOWS.bat. Run `cargo test --all-targets` first to compile and exercise the tests. No EXE included.

## Limitations
No full docking, scene tabs, true hierarchical transforms, axis-handle picking, game-camera preview or verified native rendering. Collider outlines are visual object bounds, not precise physics debugging. Partial bilingual localization. No new runtime/export functionality.

## Manual QA
Open examples/demo/project.forge.json; test GPU and fallback views, zoom/pan, search, duplicate, undo/redo, delete and reparent, snap and iso grid, animation stepping, save/reopen and Play/Stop.

---
# Forge Engine 0.2 — Tools and Runtime Preview
Native Rust / Bevy 0.16.1 / egui editor; no HTML or webview. Based on v0.1, schema 1 files load with defaults and migrate to schema 2. Keep a backup before saving.

## Windows
Install Rust stable (rustup.rs) and Visual Studio Build Tools with Desktop development with C++. Run BUILD_WINDOWS.bat. On success run dist/ForgeEngine.exe. Open examples/demo/project.forge.json, select Player, inspect Animation, press Play, use A/D and W to jump. For native runtime: `dist\ForgePlayer.exe examples\demo\project.forge.json`. `cargo test --lib` runs model tests. GitHub Actions workflow can build on Windows. No binaries or Cargo.lock are supplied: no Rust toolchain was available to compile or test this delivery.

## Added in this preview
- Shared simulation used by editor Play and standalone Bevy sprite renderer: static/dynamic AABB collisions, vertical gravity, triggers, WASD, Lua callbacks, graph traversal. Physics is intentionally elementary; no rotated colliders, dynamic/dynamic resolution or fixed-step CCD.
- Sprite atlas grid slicing, timeline scrubbing, frame count, FPS and loop; runtime frame animation. Grid dimensions are manually specified. One clip per object.
- File import via picker and egui dropped files, texture thumbnails and timestamp reload of imported copies. Assets panel lists referenced images, not a complete recursive filesystem browser. Original external source changes do not propagate to imported copies.
- Inspector parent reference (organizational only; no inherited transforms, recursive tree or cycle prevention), camera marker (first camera used by standalone runtime), move/rotate/scale drag modes and visual axes. Isometric projection and depth sorting. Grid is still rectangular.
- Lua 5.4 embedded code per object: `function update(dt) x = x + input_x * 100 * dt end`; `function on_trigger(name) end`. x,y are world coordinates; input_x,input_y are WASD axes. Instruction hook limits long-running scripts; not a security sandbox for untrusted scripts. Errors in console.
- Visual logic: add Update/Trigger event and MoveRight/MoveUp/Jump action, tick outgoing connection boxes by node index. Graph traversal prevents repeated visits on cycles. No free-position graphical node canvas or wires yet.
- Dark editor with resizable panels and bottom tabs; bilingual toolbar inherited; new controls mostly English. No detachable docking system yet.
- Editor viewport remains egui-painted (Bevy window/GPU backend), not a Bevy camera render target. Standalone player uses real Bevy sprites/camera. Camera marker not yet used in editor play preview.

## Remaining work
Compile/test on Windows; actual Bevy render-to-texture viewport; recursive hierarchy and transform inheritance; docking; proper node graph canvas; recursive asset browser and source watching; isometric grid; full localization; export pipelines. Android/iOS not provided. All functionality described is source-level implementation, not a verified running build.

## v0.3 — Native viewport and animation foundation

- GPU toggle: Bevy Camera2d renders ECS sprites into a resizable image, displayed in native egui. Disable GPU to use the v0.2 painter fallback. Grid and selection overlays remain egui. Image cache uses file modification timestamps.
- Nested scene tree with cycle-safe traversal; parent links remain organizational, not inherited transforms.
- Animation: named clips, Store clip / Load clip, atlas start frame, preview play/pause, rewind, selected-frame thumbnail. Clip selection is manual; automatic state transitions are not implemented yet.
- Refined dark theme; existing resizable panels remain. True docking is not implemented.
- Existing JSON schema 2 remains readable; new fields have serde defaults. Keep backups before opening newer projects in older editors.
- No local Rust compiler was available: source is not compiled or runtime tested. Windows build and CI scripts are retained. GPU viewport is experimental.

### Smoke test after building
1. Open examples/demo/project.forge.json. Compare GPU on/off at different zoom and pan values.
2. Select a sprite, open Animation, choose atlas dimensions and start frame, Store clip, edit and Store a second clip, then Load clip. Preview, rewind, save and reopen.
3. Check Lua/physics/visual graph from v0.2 and verify Play/Stop does not mutate the saved scene.
4. Create nested parent links, check tree, import PNG and edit the image externally to verify reload.

### Roadmap (not implemented here)
1. Harden native viewport and camera controls; 2. docking and editor icons; 3. animation state machine; 4. Aseprite JSON import; 5. Rapier physics; 6. graphical node editor; 7. tilemap studio; 8. lighting/shaders; 9. build manager (Windows then Android/iOS); 10. audio studio.

## v0.3.2 Native Editor PRO (experimental)
- Shift-click selection, group transforms, Ctrl+C/V object copies and Ctrl+G group hierarchy.
- Asset browser scans nested project folders every two seconds, filters image paths, shows thumbnails and reference counts, and creates objects from existing images.
- Clickable animation frame strip; FPS estimate and object count in status bar.
- Preferences window with configurable focus/move/rotate/scale shortcuts (session only).
- Recovery snapshot every 30 seconds for dirty saved projects. To recover, open the sibling `project.forge.recovery.json` manually; save under the original name only after reviewing it. Original project is not overwritten by autosave.
- Existing native Bevy viewport and runtime retained. No web stack.

Not implemented: free docking, separate scene tabs, marquee selection, drag reparenting, prefab files, keyframe property curves, state machine, pivot/hitbox editing, persistent preferences, full localization and memory profiler. Grouping is organizational; inherited transforms are not implemented. No compiler available here; source changes and tests are not build-verified. Windows build via BUILD_WINDOWS.bat.

## v0.3.3 Native Editor Ultimate — experimental increment

Implemented in source: independent project tabs (per-tab undo/redo and paths), workspace preferences stored under APPDATA/ForgeEngine/settings.json (or HOME), switchable side panels, viewport rulers and marquee center selection, camera bounds overlay, prefab subtree serialization with fresh UUIDs on import, motion keyframes with linear/ease interpolation, X curve visualization, normalized pivot and per-frame hitbox metadata, basic Idle/Run/Jump selection in runtime. Static non-player objects with Body=None can be animated by transform keys. Prefabs reference existing project-relative assets; cross-project resource bundling is not implemented. Camera bounds are illustrative 640×360, not a separate rendered camera feed. Hitboxes and pivot metadata are not applied to physics/rendering yet. Curve handles, true dock drag/drop, hierarchy transforms, full localization, automatic crash recovery and OS memory profiler remain incomplete. Existing recovery files and source modules retained. Scene tabs are separate project documents, not a multi-scene runtime.

Windows: run BUILD_WINDOWS.bat or use the included GitHub Actions workflow. No Rust toolchain in authoring environment; no successful compilation or GUI test is claimed. New tests in src/ultimate.rs are unexecuted.

## v0.3.4 — Editor stabilization patch (uncompiled)
- Drag a scene row onto another row to reparent, or onto the root drop area to detach. Cycle validation preserves world coordinates.
- Parent translation/rotation propagates to descendants whose transform was not independently changed. This is world-space delta inheritance, not a complete local-transform system; scale inheritance and hierarchical physics remain unsupported.
- Pivot offsets are applied in both viewport and standalone sprite rendering. Collision shapes remain unchanged.
- Drag the colored axis endpoints to choose an axis; Alt+right-click adds a pair of viewport guides; Clear guides removes them.
- Startup offers the latest marked recovery snapshot for a saved project. Recovery opens the backup in a tab; use Save As to restore under the desired name. It may offer an old snapshot even after a clean exit. Unsaved projects and other tabs are not covered.
- Windows build script now runs tests before release build. New unit tests cover reparenting, inherited translation and pivot math.
- No Rust compiler or network access in authoring environment; no compilation, tests or EXE verified. Full docking, full localization, camera inset, animated colliders, curve handles, memory profiling and robust crash recovery remain outstanding.
