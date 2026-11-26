@echo off
cd /d "%~dp0"
cargo build --no-default-features --bin forge_player --profile lightweight --target x86_64-pc-windows-msvc
exit /b %errorlevel%
