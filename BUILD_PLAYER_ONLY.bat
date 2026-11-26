@echo off
cd /d "%~dp0"
cargo build --no-default-features --features physics-rapier,lua,audio,lighting --bin forge_player --profile lightweight --target x86_64-pc-windows-msvc
if errorlevel 1 exit /b 1
echo Player: target\x86_64-pc-windows-msvc\lightweight\forge_player.exe
