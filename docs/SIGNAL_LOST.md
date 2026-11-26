# Signal Lost — v0.7 acceptance demo

Native 2D platformer; recover three energy cores and reach the exit. A/D move, W jumps, R restarts. The on-screen buttons are also wired to the existing HUD input. The scene is editable in Forge via `examples/signal_lost/project.forge.json`; objective logic is currently in the standalone player and is enabled for projects with `Core:` objects. The original demo remains available. No HTML or external media dependencies. Original generated pixel art and synthesized WAV included.

## Windows verification
1. Install Rust stable (MSVC) and Visual Studio C++ Build Tools with Windows SDK.
2. Run `BUILD_WINDOWS.bat` (from a terminal if you want to retain the console). It runs check, unit tests, headless smoke, then release build and packaging. See `forge-check.log`.
3. Launch `dist/SignalLost/PLAY.bat`. Open the project in `dist/ForgeEngine.exe` using the original project path for editing.
4. Manual acceptance: start, move, jump, land, collect each core once, reach exit, see completion, press R, verify all cores return. Listen for ambient audio, check drone movement, particles and HUD. Test controller and editor independently.

The smoke test checks settling under gravity, horizontal movement, finite coordinates, Lua errors, and collection/exit logic (the last uses direct positioning, not a full automated playthrough). It does not test GPU, audio, input devices or window creation. Demo defaults to legacy AABB physics to minimize unverified dependencies; Rapier remains available in other projects. No Windows executable is included in the source ZIP. CI has not been run here.

Deferred: GPU lighting/particles, mobile builds, advanced physics events, full visual UI designer, audio mixer and chunk batching. This release deliberately focuses on the demo and build reliability.
