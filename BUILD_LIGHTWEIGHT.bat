@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\build_windows.ps1" -Profile lightweight
set "RESULT=%ERRORLEVEL%"
pause
exit /b %RESULT%
