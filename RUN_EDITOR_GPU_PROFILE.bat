@echo off
cd /d "%~dp0"
setlocal
set FORGE_GPU_PROFILE=1
call RUN_EDITOR_DEBUG.bat
endlocal
