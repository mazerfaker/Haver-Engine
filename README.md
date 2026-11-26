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
