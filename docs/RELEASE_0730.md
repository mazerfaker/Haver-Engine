# Forge Engine v0.7.30 — Group Timeline & Asset Database

Source candidate based on the exact v0.7.29 source archive. No Auto Rig or automatic animator added. Existing runtime paths and example projects retained. No dependencies changed.

## Implemented in source
- Independent property channel editor: Ctrl+click multi-select, Select all/Clear, Copy group, Paste at playhead, Delete group, batch Smooth/Bezier interpolation. Clipboard is session-only and scoped to the channel type (X, Y, rotation, scale X/Y, opacity). Paste preserves relative times and interpolation, rejects overlapping times or timeline/capacity violations before mutation. Individual Bezier handles remain editable. Group drag/box selection and 2D Bezier handles are not implemented.
- Batch image import: project-wide exact-byte reuse, size buckets and cached content hashes during one import, byte verification before reuse, deterministic candidate ordering. Skips symlinks and hidden directories, max 16 nested directory levels, 16,384 visited entries, 256 MiB existing-candidate comparison budget. Previous 128 images / 16 MiB each / 128 MiB input limits retained. No global file deletions. All-or-nothing for newly written directory. No persistent decoded-image cache or cross-project duplicate index added.
- Asset database: sound references and composite prefab local object references; bounded directory walk (24 levels, 32,768 entries), source file size guard, symlink guard on database. Existing UUIDs preserved. This reports direct asset use per scene/prefab, not a fully resolved transitive dependency graph; invalid source files may be skipped.
- Lighting presets: Low = 1 light, hard shadow; Balanced = 2 lights, 6 units softness; High = 3 lights, 16 units softness. Existing sliders remain available. Presets change existing settings only; do not enable GPU lighting automatically. Shadow candidate query expands for soft shadow endpoint displacement. Existing shaders unchanged.
- Prefab graph: automatic layout load on opening a different composition and save on drag release, with opt-out and manual buttons retained. No write if content is unchanged; refuses symlink layout file. Recursive dependency tree refreshes when composition references change, selected dependency highlighted. Does not provide freely editable recursive graph nodes or live monitoring external file changes.
- Profiler: CPU lighting synchronization duration including index time and material/transform updates; exported as `lighting_sync_cpu_ns`. Zero on idle path. GPU timestamps and total VRAM remain null/unavailable; no fabricated timings. Previous RSS, UI frame sampling, index stats, estimates preserved.
- Global style: unified body/button type sizes and icon spacing. Full redesign of all panels is not complete.

## Windows acceptance tests (not executed here)
1. Extract patch over v0.7.29 project; keep target and Cargo.lock. QUICK_CHECK.bat then QUICK_TEST.bat. Send `forge-quick-check.log` and `forge-quick-test.log` if failure. Existing lock preparation migrates root package only; does not change dependency pins.
2. QUICK_BUILD.bat; RUN_EDITOR_DEBUG.bat. Open `examples/visual_channels/project.forge.json`. Select channel keys using Ctrl+click, copy, move playhead to an empty interval, paste. Confirm overlaps are rejected, group deletion/curves work, save/load and undo work.
3. Batch import the same PNG twice in separate operations; only one stored copy. Inspect asset database with scene sound and composite prefab references. Confirm invalid batch does not delete existing files.
4. RUN_GPU_SCENE.bat; compare three profiles via editor Gameplay settings. Ensure off/isometric fallback unchanged. Validate shadow edges visually.
5. Open existing composite from `examples/frame_geometry`; drag reference nodes, close and reopen graph/editor, verify layout restoration. Test cyclic reference rejection and external edit conflict protection.
6. Export profiler JSON; confirm CPU sync nonnegative and GPU time/total VRAM null. Compare matched scenes, same resolution and hardware; no FPS claims before measurement.
7. CHECK_EXPERIMENTS.bat and optionally GPU_INSTANCING_LAB.bat, AUDIO_DSP_LAB.bat. No new runtime instancing integration.

## QA status
Python checks and archive reconstruction are static checks, not Rust execution. Rust compiler and cargo unavailable in authoring environment. New Rust tests have not run; Windows build and GPU/audio runtime remain unverified. Do not treat this as a stable binary release.
