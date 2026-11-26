# Forge Engine 0.7.18 — UI Studio & Workspace

## Baseline
Exact v0.7.17 source archive, which already includes the v0.7.16 GPU Particle Lab hotfix. v0.7.17 compilation and integrated GPU runtime are not yet confirmed. Existing shaders and GPU implementations are unchanged. No dependency changes, no Auto Rig or automatic animator.

## New source implementations
- Dedicated UI Studio window accessible from the main toolbar while not playing. Group tree (up to eight levels), direct selection and dragging in a resolution-aware canvas, 5% optional snapping, per-widget text/action/group/size/anchor editing, duplication, list order changes and deletion. Drag uses existing history transaction; other edits use existing frame-level undo. Preview presets: project, 16:9, 4:3, portrait.
- Explicit opt-in `hud_canvas_groups` scene flag (default false): grouped widgets render individually at adaptive anchors in standalone Player; legacy group-window behavior remains unchanged unless enabled. Groups remain logical organization, not geometric nested containers. Supports existing left/right/jump actions and labels. Inventory is a six-label layout template only, not item storage or game logic. Preview is not a live Player renderer; fonts and widget styling may differ. No general main-menu action system.
- Workspace presets: 2D, Animation, Assets (top), Wide viewport (tabbed Scene/Inspector on right). They save existing settings and docked/floating state. Not arbitrary split-tree docking or a full redesign.
- `RUN_UI_STUDIO_SCENE.bat` opens six-widget fixture; Windows build script includes new editor model QA fixture. Three new Rust tests for path normalization, drag bounds, legacy HUD defaults. Existing GPU and campaign tests retained.
- Local package version migration supports v0.7.17 (and older supported versions) while preserving dependency pins. ZIP update does not include Cargo.lock or target.

## Verification
Static only: existing campaign assets/atlas checks, asset pack tests, GPU fixture tests, UI fixture/reference checks, JSON/TOML parsing, archive integrity and patch reconstruction. Rust compiler and Windows GPU execution unavailable here; new Rust tests and rendering not run. No verified performance gain or EXE. No new DSP/audio effects, arbitrary dock splits, instanced GPU particles, or GPU timing telemetry in this increment.

## Test on Windows
1. Back up current source and known-good v0.7.15 portable. Apply update to v0.7.17 source folder, keep Cargo.lock and target.
2. Run QUICK_CHECK.bat (includes GPU examples), QUICK_BUILD.bat and RUN_EDITOR_DEBUG.bat. Verify previous GPU fixtures with RUN_GPU_SCENE.bat and RUN_GPU_PARTICLES_SCENE.bat, since 0.7.17 was not tested.
3. Run RUN_UI_STUDIO_SCENE.bat; open examples/ui_studio/project.forge.json in Editor. Click UI Studio. Try all preview ratios, drag a control, undo, change group, duplicate, reorder, save/reopen. Toggle canvas mode and check Player. Try workspace presets. Test actual controls in standalone Player.
4. Run BUILD_WINDOWS.bat after these succeed; send forge-quick-check.log or forge-check.log if issues arise.
