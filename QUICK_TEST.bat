@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File tools\quick_test.ps1
set "code=%errorlevel%"
pause
exit /b %code%
