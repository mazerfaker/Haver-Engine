@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File tools\prepare_lock.ps1
if errorlevel 1 goto end
cargo check --locked --no-default-features --features gpu-instancing-lab,audio-dsp-lab --example gpu_instancing_lab --example audio_dsp_lab --target x86_64-pc-windows-msvc
:end
pause
