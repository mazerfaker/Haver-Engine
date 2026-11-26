@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\build_windows.ps1"
set "RESULT=%ERRORLEVEL%"
echo See forge-check.log for diagnostics.
pause
exit /b %RESULT%
