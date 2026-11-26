# v0.7.27 — Live DSP & Property Channels (source candidate)

See `docs/RELEASE_0727.md` for changes, limitations and Windows validation.

# Forge Engine v0.7.26 — Studio Refinement & Optional DSP

See [release notes](docs/RELEASE_0726.md). Run QUICK_CHECK.bat and QUICK_TEST.bat before release. GPU instancing remains an opt-in lab: CHECK_EXPERIMENTS.bat. RUN_AUDIO_RUNTIME_SCENE.bat tests optional PCM16 playback DSP. Source only; Rust and GPU/audio execution not verified.

# Forge Engine v0.7.11 — Combat & UI Refinement

See [release notes](docs/RELEASE_0711.md) and `QA_STATIC_0711.txt`. Source only; compilation not verified.

# Forge Engine v0.7.10

See [release notes](docs/RELEASE_0710.md). J attacks in the demo.

# Forge Engine v0.7.8 — Modular Runtime

See [changes and limitations](docs/PERFORMANCE_077.md). Run `CHECK_WINDOWS.bat` before building. For player-only build use `BUILD_PLAYER_ONLY.bat`.

# Forge Engine v0.7.6 — Professional & Performance

Experimental source update. See [performance report](docs/PERFORMANCE_076.md). Start with CHECK_WINDOWS.bat. No compiled binaries supplied.

# Forge Engine v0.5 — Gameplay PRO

Experimental source update. See [Gameplay PRO](docs/GAMEPLAY_PRO.md) and [platform export](docs/PLATFORM_EXPORT.md). Previous documentation below is retained for legacy features.

# Forge Engine v0.4.0 — Gameplay Foundation

Experimental native Rust/Bevy/egui source update based on v0.3.7. No compiled executable or verified build.

## New tools
- Tilemap tab: enable Paint in scene, choose grid step, layer, Solid and optional project-relative PNG path (`assets/...`). Left click/drag in viewport; Eraser removes the matching cell. Tiles are individual scene objects, persisted by normal save and rendered in both editor and player. No atlas palette or batching yet.
- Visual Script tab: add nodes, drag their headers, click an output then an input to toggle a link. IfRight/IfJump gate outgoing edges. Layout is session-only; links and nodes persist. Execution visits each node once per event, not a general-purpose branching language.
- Inspector: gravity multiplier, movement speed, jump speed, bounce. Physics is the existing axis-aligned solver, not Rapier; no rotated colliders, friction or dynamic-body collision. Shared Rust raycast helper tests static AABBs; not exposed in Lua/UI yet.
- Lua: existing update(dt), on_trigger(name), x/y/input_x/input_y plus elapsed, object_name, velocity_y, move(dx,dy), get_position(), jump(). The helpers are available before update, not script initialization.
- Default new project now has a dynamic player and static platforms.

## Validation
Run CHECK_WINDOWS.bat and BUILD_WINDOWS.bat after installing Rust and C++ Build Tools. Existing CI workflow runs tests and builds Windows binaries. New tests cover tile replacement/erase, raycast and graph conditions. Cargo and rustc are unavailable in the authoring environment; tests and compilation were not executed. Rapier, full atlas editor, persisted graph layout, full Lua SDK, exports and full localization remain pending.

## Previous documentation (v0.3.7)

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

## v0.3.5 Professional Workspace

- Panels menu: Scene, Inspector and Tools can be detached into movable/resizable **in-app** windows, or returned to their existing dock positions. Save layout persists window rectangles and mode flags. Full tab docking and separate OS windows are not implemented.
- Projects: recent files, empty 2D, platformer and isometric templates; opening checks unsaved changes.
- Inspector: right-click Transform to reset fields; Add Component enables existing camera, physics and Lua components.
- Assets: folder selection, search, PNG/JPEG filter, thumbnail grid, double-click to instantiate, drag a thumbnail to the viewport. Existing asset scan and change tracking remain.
- Viewport: rotation ring and angular pointer delta, existing move/scale axis handles. Camera preview uses a second Bevy render target (640x360), first Camera2D object, current scene or play state.
- Animation: X/Y/Angle keyframe tracks and draggable playhead; existing motion inspector provides numeric key editing and sampled curve. Independent curve handles and key dragging are not yet implemented.
- UA/EN: new workspace controls and several existing toolbar labels translated; legacy controls and diagnostics remain partly English. No complete professional SVG icon set yet.
- Existing recovery and Windows CI retained. No successful compilation or interactive testing: cargo/rustc absent and network DNS for Rust distribution unavailable. Source only, no EXE. All listed features are source implementations, not verified runtime behavior.

### Windows validation
Run `CHECK_WINDOWS.bat` first. It logs compiler/test output to `forge-check.log`. Then run `BUILD_WINDOWS.bat` if checks pass. GitHub Actions workflow is also included for a clean Windows build.

## v0.3.6 — Editor Master (experimental source)

Studio menu: Dark, Light and OLED theme; accent picker; persistent settings; left/right/bottom panel placement; local gizmo axes; profiler and Game Preview PRO. Existing floating panel controls remain in Panels. Game preview uses the Bevy render target with configurable dimensions, pause and fixed 1/60-second step. Add a Camera2D first. Animation tab includes interactive X/Y/rotation keyframe graph; drag a key to change time/value; smooth checkbox uses cubic smoothstep, not editable Bezier tangents. Assets: click a filename to select, import a folder, or specify an existing destination relative to assets/. Rename/move restricted to a saved single document with empty history; save immediately afterward. References in other project files/prefabs and historical snapshots are not migrated. Prefab editor opens a separate document from a prefab JSON and provides Save prefab document; texture resolution and nested prefab dependencies remain limited. Profiler reports estimated egui texture bytes, not total process or GPU memory. Full free-form docking, custom vector icon system, editable Bezier handles, full cross-document asset refactoring, and Windows compile/run verification remain pending. The previous source is retained in its own archive. No EXE is included. Run CHECK_WINDOWS.bat then BUILD_WINDOWS.bat; GitHub Actions can perform Windows compilation and tests.

## v0.3.7 — Native Editor MAX (experimental source)
- MAX menu: object-center snapping, asset index and profiler, game diagnostics. Drag panel labels in the scene toolbar onto Left/Right/Bottom targets. Existing floating panels remain available; arbitrary nested split trees and grouped dock tabs are not implemented.
- Assets: Import Aseprite JSON Array. Export PNG beside JSON, equal-size untrimmed/unrotated frames. Forward, reverse and ping-pong tags are expanded into a normalized grid; per-frame milliseconds are stored in animation data and sampled by the runtime. Existing fixed-FPS clips remain supported. No live Aseprite integration or JSON Hash import.
- Refresh database writes `.forge-assets.json` with path-associated UUIDs, size, presence and references found in saved `*.forge.json`, `*.prefab.json` and active scene. UUIDs are not yet stable across renames; dependencies outside those files and other unsaved tabs are not indexed. The index does not rewrite project references.
- Bezier controls in Animation: two vertical handles per outgoing segment, separately for X, Y, Rotation. Horizontal handle locations are fixed at 1/3 and 2/3; this is not a full two-dimensional tangent editor. Old smooth/linear keys load unchanged.
- Debug: selected object's runtime transform, body, clip, recent trigger-enter events and Lua errors. No arbitrary Lua globals inspector or breakpoint debugger.
- Profiler: process resident memory using sysinfo; recent egui frame intervals, not GPU timestamps. New vector Forge mark in toolbar and branding/forge.svg. Not a complete icon system. New primary controls bilingual; inherited UI remains partially localized.
- Static fixes: avoid inserting hitboxes simply by opening inspector; hide preferences unless requested; tighten viewport sprite query exclusions. No compiler or Windows toolchain in build environment; dependency resolution, compilation and runtime are unverified. `CHECK_WINDOWS.bat` and CI remain the validation path.

## v0.6 Production Tools
See [implementation status and limitations](docs/PRODUCTION_TOOLS.md). Use Production menu for build resolution, input, joints; object inspector for NPC navigation and particle texture. Source-only experimental release, compilation not verified.

## v0.7 — Demo & Build focus
New editable Signal Lost demo: `examples/signal_lost/project.forge.json`. Use `PLAY_DEMO.bat` or run `BUILD_WINDOWS.bat` to check/test/build/package. See [acceptance checklist](docs/SIGNAL_LOST.md). Source only: Rust compiler and Windows runner are unavailable in the authoring environment. The preceding README sections describe retained historical features, not newly verified functionality.

## v0.7.3 — Demo & Editor QA consolidated update

- Signal Lost PRO: three editable sector JSON files in `examples/signal_lost`, new pixel backgrounds and enemy art, patrol enemies, three health points, damage cooldown, menu and retry, sector progression, collect/hurt/win WAV effects. Use A/D, W, Esc, Enter, R, N. The original v0.7 archive is unchanged. The first sector is `project.forge.json`; start it for the campaign. Opening another level directly plays it independently.
- Editor Play mode uses the same core/health rules and displays a compact status. Campaign transitions and audio cues are standalone-only. Save As now copies referenced sound/particle/normal map paths as well as sprites. Standalone renderer removes obsolete entities after scene changes.
- New `--editor-qa` model checks and all-sector smoke gates. See `docs/EDITOR_QA_073.md` for manual tests and limitations.
- `python tools/validate_demo.py` validates all three sector data files and cues. No Rust toolchain is available in the authoring environment, so no compilation, unit test execution, Windows EXE or GUI QA is claimed. All new Rust features remain experimental until build verification.

## v0.7.4 — Windows build and packaging hotfix
See [Windows build guide](docs/WINDOWS_BUILD.md). The old wildcard copy of multiple campaign JSON files to one destination is replaced by explicit file copies. The local and CI pipelines now share one PowerShell script, preserve Cargo.lock, stop on failing commands, and stage portable artifacts before publication. Rust/PowerShell unavailable in authoring environment; no compiled EXE, no actual compiler log, no confirmed runtime execution. Previous gameplay and editor source preserved except version label.

## v0.7.5 — Editor Polish
Wrapped toolbars, scrollable inspector, Frame Scene (Home), selection counter, bilingual shortcut help, selection cleanup, recent-project ordering, axis reset and play-mode drag guards. Existing runtime and demo assets unchanged. Rust compilation unavailable in authoring environment; run CHECK_WINDOWS.bat before BUILD_WINDOWS.bat. See docs/EDITOR_POLISH_075.md.

## v0.7.8
See [performance report](docs/PERFORMANCE_078.md). Minimal player: BUILD_MINIMAL_PLAYER.bat.

## v0.7.9
See [Engine Refinement and limitations](docs/PERFORMANCE_079.md). Run QA_MATRIX.bat before Windows packaging. Asset packing: tools/pack_assets.py.

## v0.7.12
See [release notes](docs/RELEASE_0712.md). Experimental source; no compiled EXE.

## v0.7.13 — Professional Editor & Engine
See [release notes](docs/RELEASE_0713.md). Added top docking and draggable vector headers, distance-based SFX attenuation, named HUD groups, CPU viewport culling and optional GPU lighting lab (`GPU_LIGHTING_LAB.bat`). Source-only, compilation unverified. Auto Rig excluded.

## v0.7.14 — Runtime & Workspace maintenance
- Asset Browser: pages of 48 thumbnails, at most four decodes per UI frame, page-scoped thumbnail cache.
- HUD: slash-separated container paths (e.g. HUD/Inventory/Equipment) in Player; maximum nesting 8.
- Mixer: one source index per mixer update and avoid redundant volume writes.
- Workspace: explicit Save dock command; existing four docking edges and floating panels retained.
- GPU lab and CPU fallback unchanged. No Auto Rig.
- Windows compilation and graphical testing remain unverified; see docs/RELEASE_0714.md.

## v0.7.15 — Stability
See [release notes](docs/RELEASE_0715.md) and [Windows build guide](docs/WINDOWS_BUILD.md). Includes v0.7.14 level-four hotfix, explicit f32 typing at compiler-diagnosed locations, feature-specific warning cleanup, scene deserialization regression tests, active cache metrics, and UTF-8 build logs. Run CHECK_WINDOWS.bat, then BUILD_WINDOWS.bat. GPU_CHECK.bat checks the opt-in lab Rust target; actual WGSL rendering still needs a graphical run.

## v0.7.16 — GPU & Editor Integration
See [release report and Windows QA](docs/RELEASE_0716.md). Default CPU lighting remains active for legacy scenes; enable experimental GPU lighting in Gameplay PRO. `RUN_GPU_SCENE.bat` opens a small normal-map and moving-light test scene; `GPU_PARTICLE_LAB.bat` opens an isolated compute test. `QUICK_CHECK.bat`, `QUICK_BUILD.bat`, and `RUN_EDITOR_DEBUG.bat` reuse the existing source directory's Cargo cache. Keep your Cargo.lock and target. New version is uncompiled source, not a verified Windows release.


## v0.7.17 — GPU particle runtime and editor refinements
See [release notes](docs/RELEASE_0717.md). Start with QUICK_CHECK.bat, then QUICK_BUILD.bat. Open examples/gpu_particles/project.forge.json in the editor, or use RUN_GPU_PARTICLES_SCENE.bat. Existing scenes use previous defaults. GPU runtime is experimental; no Windows build was performed in the authoring environment.

## v0.7.18 — UI Studio & Workspace
Open **UI Studio** in the editor toolbar (outside Play) for hierarchy, responsive layout previews, anchors, and group editing. Enable canvas placement for grouped HUD explicitly to render group members at their anchors in Player; old scenes keep their existing group windows. `RUN_UI_STUDIO_SCENE.bat` launches the example; Studio menu contains workspace presets. See `docs/RELEASE_0718.md` for limitations and validation steps.

## v0.7.20 — Animation & Scene Studio

See `docs/RELEASE_0719.md`. `RUN_STUDIO_PRO_SCENE.bat` opens the new fixture. Keep Cargo.lock and target when updating.

## v0.7.20 — Docking, Graph & Atlas (experimental source)

See `docs/RELEASE_0720.md` for controls, testing and limitations. Optional recursive docking and vertex-quad particles preserve the legacy default paths. Use `QUICK_CHECK.bat` before building, and `RUN_GPU_QUADS_SCENE.bat` / `RUN_ATLAS_SCENE.bat` for the new fixtures. No dependency changes. Keep your existing `Cargo.lock` and `target`.

## v0.7.21 — Studio Refinement & Runtime Safety

See `docs/RELEASE_0721.md`. New atlas rectangle editor, tab reordering, scene audio lifetime fixes and independent linked prefab overrides. Run `QUICK_CHECK.bat` and `QUICK_TEST.bat` before building. Source release, not yet compiled on Windows.


## v0.7.22 — Inventory & Animation Events
See [release notes](docs/RELEASE_0722.md). Run QUICK_CHECK.bat, QUICK_TEST.bat and RUN_INVENTORY_SCENE.bat.

## v0.7.23 — Inventory PRO & Checkpoints
See [release notes and Windows checks](docs/RELEASE_0723.md). `RUN_INVENTORY_PRO_SCENE.bat` exercises item effects, equipment and checkpoints (F5/F9). The checkpoint lives beside the entry scene, not inside the authored scene file. The new source is not yet a confirmed compiled release; preserve your previous working version.

## v0.7.26 — Timeline & Frame Geometry

New transform-key timeline, per-frame trim geometry independent of collider size, external nested prefab loader, shared docking icons and small CPU update optimizations. See `docs/RELEASE_0724.md`. Run `QUICK_CHECK.bat`, `QUICK_TEST.bat` and `RUN_FRAME_GEOMETRY_SCENE.bat`. Rust compilation and GPU runtime still require Windows validation. Hardware instancing and live DSP remain unfinished.
