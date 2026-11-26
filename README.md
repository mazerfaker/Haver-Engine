# Forge Engine v0.1 — native editor prototype

Windows-first Rust / Bevy 0.16.1 / bevy_egui 0.35.1. No HTML, WebView or Electron.

## Windows
1. Install Visual Studio 2022 Build Tools with Desktop development with C++ and Windows SDK.
2. Install stable Rust (MSVC) from https://rustup.rs/. Restart the terminal.
3. Extract the ZIP, run BUILD_WINDOWS.bat. The script builds both binaries in dist/. Internet is needed for Cargo dependencies.
4. Run dist/ForgeEngine.exe; open examples/demo/project.forge.json.

Alternatively RUN_EDITOR.bat starts a debug build. From PowerShell, run `cargo test --lib` and `cargo run --bin forge_player -- examples/demo/project.forge.json` to test standalone gameplay. To use the compiled player: `& .\dist\ForgePlayer.exe .\examples\demo\project.forge.json`.

## Controls
- Left click selects, left drag moves, middle drag pans, wheel zooms.
- Snap 16 rounds position at the end of a drag. Inspector edits size, angle, layer, color, name and WASD control.
- Ctrl+S saves, Ctrl+Z/Y undo/redo, Delete removes selection (when not editing text).
- Play tests movement in an isolated copy of the scene; Stop discards play changes. Standalone player uses Bevy sprites/ECS. Preview is an egui scene drawing, not yet a render-to-texture Bevy runtime.
- Import PNG/JPG copies the file to the project assets folder and creates a sprite object. Save the project first. Save As copies referenced assets; existing files at matching paths can be overwritten.
- UA/EN switches primary interface labels; some technical labels and dialogs remain English.

## Architecture and portability
`src/lib.rs` is the shared versioned JSON model; `forge_editor` and `forge_player` are independent binaries. The JSON stores stable UUIDs, Y-up coordinates, pixels, degrees, layer ordering, relative texture paths and a simple controller flag. Keep assets and JSON together. No engine handles are serialized. Later adapters must translate coordinate systems, behavior, animation, physics and rendering. Current v0.1 is a flat single scene; no hierarchy, camera editing or isometric tools yet.

## Status / known limitations
Source prototype only. Rust and Windows toolchain unavailable in authoring environment, so no compile, runtime or GPU verification was performed. The included GitHub Actions workflow has not been run. No Cargo.lock yet (generated on first build). Close-window unsaved confirmation, drag-and-drop files, full localization, animation, Lua, visual scripting, physics, mobile builds and Godot/Unity exporters are not implemented. No production readiness claim.

## Next version choices
A. Animation editor + sprite sheet slicing + asset browser.
B. True Bevy render-to-texture viewport + transform gizmos + hierarchy.
C. Lua component scripting + behavior lifecycle. Recommended: B then A then C.
