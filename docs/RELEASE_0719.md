# Forge Engine v0.7.19 — Animation & Scene Studio

Base: exact published v0.7.18 archive. Existing projects, dependencies, WGSL shaders and CPU fallbacks retained. No Auto Rig or automatic animation authoring.

## Implemented in source (not yet compiled)
- Animation Studio: explicit ordered clip transitions (Idle, Moving, JumpInput, Finished, VariablePositive), editable rules, duration per frame, variable-duration frame stepping, timeline duration, click/drag pivot preview. Existing clips, Aseprite, uniform sheet slicing retained. FSM uses player movement input only for player objects; other objects can use variable and completion rules. Save edited timing with Store clip. Rule list, not a node graph.
- Prefab library: save selected subtree under project/prefabs, insert with fresh IDs and offset; validate missing/duplicate IDs, cycles, transforms, sprite/normal/particle paths. Nested object hierarchies, not live linked nested prefab assets. Library limited to 256 listed files. Source scene assets are still referenced by relative paths.
- UI Studio/Player: pause/resume button and progress widgets bound to named player variable (0–1), canvas-mode fixture; generic Escape toggle; previous HUD layout retained. Not a gameplay inventory implementation.
- Lighting: precompute rotated blocker AABBs once per sync, compute strength once per candidate, same sort/selection and unchanged shader. No new shadow algorithm.
- Particles: optional per-small-emitter cap 256, default 128; larger quads capped at 128, existing 8 emitter cap; no instanced rendering, existing per-fragment loop retained. Raising count can increase GPU cost; not benchmarked.
- Audio: smoothing of live mixer gain except initial or zero gain. Existing distance attenuation and pitch retained. No new stereo panning, EQ or reverb.
- Editor: saved compact/comfortable spacing option; existing tabs, floating zones and workspace retained. Arbitrary nested splits and comprehensive redesign not implemented.
- QA: new Rust tests for timing, rules, legacy defaults, nested prefab remap/cycle, optional particle cap and studio fixture; Windows build includes editor model QA for new scene. Asset path validation for particle textures.

## Windows verification
1. Back up v0.7.18; extract update into project root, keep Cargo.lock and target. Lock migration changes root package version only; no dependency changes.
2. QUICK_CHECK.bat (includes GPU examples), QUICK_BUILD.bat, RUN_EDITOR_DEBUG.bat.
3. RUN_STUDIO_PRO_SCENE.bat: A/D changes Idle/Run, progress bar reads energy=0.65, Pause/Resume toggles simulation. Open examples/studio_pro/project.forge.json in editor, select player and Animation panel; edit transitions and pivot.
4. RUN_GPU_SCENE.bat and RUN_GPU_PARTICLES_SCENE.bat; verify previous graphics, optionally raise limit on small emitters. GPU_PARTICLE_LAB.bat for previous lab.
5. BUILD_WINDOWS.bat when checks pass. No Rust compiler or Windows runtime in this environment. No new FPS, RAM, VRAM or binary-size measurements. v0.7.17–19 not confirmed on Windows; v0.7.15 is last confirmed full Release.
