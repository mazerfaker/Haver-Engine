@echo off
cd /d "%~dp0"
cargo run --release --bin forge_player -- examples/signal_lost/project.forge.json
if errorlevel 1 pause
