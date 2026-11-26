@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File tools\prepare_lock.ps1
if errorlevel 1 goto end
cargo run --locked --bin forge_player --target x86_64-pc-windows-msvc -- examples/bus_dsp/project.forge.json
:end
pause
