@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File tools\prepare_lock.ps1
if errorlevel 1 goto end
cargo run --locked --no-default-features --features gpu-lab --example gpu_particle_lab --target x86_64-pc-windows-msvc
:end
pause
