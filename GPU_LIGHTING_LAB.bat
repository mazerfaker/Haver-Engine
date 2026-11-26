@echo off
cd /d "%~dp0"
cargo run --release --no-default-features --features gpu-lab --example gpu_lighting_lab
pause
