@echo off
cd /d "%~dp0"
where cargo >nul 2>&1
if errorlevel 1 (
 echo Install Rust from https://rustup.rs/ and Visual Studio C++ Build Tools.
 pause
 exit /b 1
)
cargo test --all-targets
if errorlevel 1 (
 echo Tests or compilation failed.
 pause
 exit /b 1
)
cargo build --release --bins
if errorlevel 1 (
 echo Build failed. See errors above.
 pause
 exit /b 1
)
if not exist dist mkdir dist
copy /Y target\release\forge_editor.exe dist\ForgeEngine.exe
copy /Y target\release\forge_player.exe dist\ForgePlayer.exe
echo Build complete: dist\ForgeEngine.exe
pause
