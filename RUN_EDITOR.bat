@echo off
cd /d "%~dp0"
cargo run --bin forge_editor
if errorlevel 1 pause
