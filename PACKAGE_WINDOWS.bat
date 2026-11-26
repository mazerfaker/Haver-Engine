@echo off
setlocal
cd /d "%~dp0"
if "%~1"=="" (echo Usage: PACKAGE_WINDOWS.bat path-to-exported-bundle & exit /b 1)
if not exist "%~1\project.forge.json" (echo Missing project.forge.json & exit /b 1)
if not exist dist\BUILD_OK.txt (echo No verified build. Run BUILD_WINDOWS.bat & exit /b 1)
if not exist dist\HaverPlayer.exe (echo Build first with BUILD_WINDOWS.bat & exit /b 1)
copy /Y dist\HaverPlayer.exe "%~1\HaverPlayer.exe" || exit /b 1
> "%~1\PLAY.bat" echo @echo off
>> "%~1\PLAY.bat" echo cd /d "%%~dp0"
>> "%~1\PLAY.bat" echo HaverPlayer.exe project.forge.json
