@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File tools\prepare_lock.ps1
if errorlevel 1 goto end
cargo build --locked --bins --target x86_64-pc-windows-msvc
:end
pause
