# Forge Engine v0.7.5 — Windows build

Status: source release, not a verified Windows binary. Windows 10/11 x64.

## Build from source
1. Install Rust using rustup.rs (stable MSVC) and Visual Studio Build Tools with Desktop development with C++ and Windows SDK.
2. In a new terminal run `rustup target add x86_64-pc-windows-msvc`.
3. Run BUILD_WINDOWS.bat from the extracted source directory. Internet access is needed on the first build to resolve/download dependencies.
4. On success, use `dist/START_EDITOR.bat` or `dist/SignalLost/PLAY.bat`. The portable ZIP is generated next to BUILD_WINDOWS.bat. Copy/extract the entire archive, not only an EXE. Windows runtime/graphics drivers may be required.
5. On failure send `forge-check.log`. CHECK_WINDOWS.bat performs the same debug check/test and headless QA gates without creating release binaries. No GUI launch is validated automatically.

The build checks each of the three distinct campaign scenes, stages release binaries and assets, checks the staged player, and only then publishes a ZIP. Cargo.lock is generated if missing and retained for subsequent builds. Do not delete it unless intentionally updating dependencies.

## Alternative: GitHub Actions
Upload the contents of the source folder (including .github) to your repository, open Actions > Forge Windows verification > Run workflow. Download the portable artifact only if the job succeeds; the log artifact is also uploaded on failure. The workflow is included but has not been executed here.

## Manual Windows acceptance
- Run START_EDITOR.bat; open SignalLost/project.forge.json through the project manager.
- Verify viewport, selection, move/rotate/scale, undo/redo, import and save/load on a copy.
- Play through all three levels; confirm health, audio, transitions and restart.
- Verify startup from a folder with spaces and without the Rust toolchain installed (on a separate machine).
- Send a screenshot or log if startup fails. Editor preferences still use the OS profile; portable means no installer, not zero writes outside the folder.
