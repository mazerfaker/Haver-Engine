@echo off
setlocal
cd /d "%~dp0"
call CHECK_WINDOWS.bat || goto failed
cargo build --release --bins >>forge-check.log 2>&1 || goto failed
if not exist dist mkdir dist
copy /Y target\release\forge_editor.exe dist\ForgeEngine.exe >nul || goto failed
copy /Y target\release\forge_player.exe dist\ForgePlayer.exe >nul || goto failed
if not exist dist\SignalLost mkdir dist\SignalLost
copy /Y dist\ForgePlayer.exe dist\SignalLost\ForgePlayer.exe >nul || goto failed
copy /Y examples\signal_lost\project.forge.json dist\SignalLost\project.forge.json >nul || goto failed
xcopy examples\signal_lost\assets dist\SignalLost\assets\ /E /I /Y >nul || goto failed
copy /Y RUN_GAME.bat dist\SignalLost\PLAY.bat >nul || goto failed
echo PASS: dist\ForgeEngine.exe and dist\SignalLost\PLAY.bat
pause
exit /b 0
:failed
type forge-check.log
echo BUILD FAILED. No successful build confirmed.
pause
exit /b 1
