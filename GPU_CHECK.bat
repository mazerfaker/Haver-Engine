@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\check_gpu.ps1"
set "RESULT=%ERRORLEVEL%"
echo See forge-gpu-check.log. A successful Cargo check does not validate WGSL rendering.
pause
exit /b %RESULT%
