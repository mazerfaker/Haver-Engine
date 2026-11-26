# Forge Engine v0.7.24 — Timeline & Frame Geometry

Base: exact v0.7.23 source archive. This is a source-only experimental update, not a compiled Windows release. No new Cargo dependencies; no Auto Rig or automatic animation authoring.

## Implemented source changes

- **Animation Timeline:** draggable transform-key diamonds, selection, seek, add/delete, manual position/rotation/time editing, fixed adjustable timeline span, Fit, linear/smooth/Bezier interpolation and per-axis Bezier controls. Existing event track, explicit clip rules, and graph are unchanged. Transform tracks remain object-global, not independent per-clip channels. Motion sampling now finds adjacent keys in one pass without allocating/sorting a temporary vector. Duplicate times preserve the original stable ordering.
- **Sprite Studio:** optional `Animation.frame_geometry` map keyed by absolute atlas frame index; normalized top-left XYWH relative to the original logical sprite. `Trim + preserve per-frame geometry` composes repeated crops. Object size, collider and global pivot are not modified. `Edit logical frame rect` and `Reset frame geometry` control the rendering rectangle. Store clip after editing. Player, native editor viewport, fallback editor canvas, and GPU lighting material all use the same geometry calculation. Old project JSON defaults to empty metadata. Atlas region edits retain the logical frame rectangle; use Reset to restore full logical size. Per-frame geometry does not change physics, hitboxes or the source PNG. Existing packing remains intact.
- **Nested Prefabs:** library reads both legacy object arrays and composite documents with `objects` and `nested`. Nested references have explicit stable IDs; expansion remaps child source IDs deterministically, preserves parent relationships and applies position/key offsets. Files must be inside the project's `prefabs` directory. Depth ≤ 8, ≤ 128 file reads, ≤ 8 MiB/file, ≤ 10,000 objects, cycle and duplicate checks. Existing linked-instance refresh and Refresh + new children now resolve the external files. Changes to membership that delete or reparent existing objects are still refused. Nested composition is authored in JSON, not in a full graphical prefab editor.
- **Editor:** shared vector icons used for both classic panel headers and recursive docking tabs; dark panel palette adjusted; tiny dock splits skipped rather than creating negative rectangles. Old workspace remains available.
- **Lighting/Performance:** skip square root for lights outside radius; skip unchanged GPU light visual transform updates; no shader changes. Motion sampling avoids temporary heap allocations/sorting. No measured FPS, RAM, VRAM, or EXE improvements claimed.
- **Build:** version 0.7.24, Windows packaging and CI names updated. Lock migration accepts 0.7.14–0.7.23 and changes only forge_engine's root package version. The archives do not contain Cargo.lock or target.

## Test on Windows

1. Back up the existing v0.7.23 source. Extract the update into the existing project root, not a new nested folder. Do not delete Cargo.lock or target.
2. Run QUICK_CHECK.bat and QUICK_TEST.bat. Send logs if anything fails; no release build needed first.
3. Run RUN_FRAME_GEOMETRY_SCENE.bat. The QA actor alternates once per second between full and trimmed UVs. Its visible pixels should remain in the same location and at the same scale. WASD should move the actor; the scene enables GPU lighting. Also switch off GPU lighting and check both editor view modes.
4. Open examples/frame_geometry/project.forge.json in Editor. Open the animation tools, Transform timeline, add and drag a key, and try Linear/Smooth/Bezier. In Prefab library click parent.prefab.json, which should create a parent and one nested child offset 32 units. Change child color in the source JSON and refresh the linked instance. Check appearance override, and test adding a new nested child with Refresh + new children.
5. Regression: RUN_GPU_QUADS_SCENE.bat, RUN_INVENTORY_PRO_SCENE.bat and CHECK_WINDOWS.bat.

## External nested prefab format

The file `examples/frame_geometry/prefabs/parent.prefab.json` demonstrates the format. Each nested reference must keep its `id` UUID unchanged across edits. `parent` points to a local object UUID, or may be omitted for a new root. `offset` is a world-space XY translation, default [0,0]. Referenced filenames are relative to the `prefabs` directory and must end in `.prefab.json`. Standalone scenes retain materialized objects. External assets must still be valid project-relative paths.

## Not completed / not verified

True hardware GPU instancing is not implemented; previous indexed GPU quads and fallback are retained. Real-time EQ/Reverb/DSP is not implemented; previous spatial audio/offline WAV effects remain. A comprehensive editor redesign, independent per-clip transform channels, new shadow algorithm, deletion/reparenting merge for prefabs, and measured performance gains are not complete. Rust/Cargo and a Windows/GPU runtime were unavailable in this environment. New Rust tests and WGSL execution have not been verified; the last confirmed full Release remains v0.7.15 based on earlier user testing.
