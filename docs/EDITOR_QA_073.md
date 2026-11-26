# Editor QA / 0.7.3

## Run on Windows
From the repository directory run CHECK_WINDOWS.bat (Rust stable MSVC + C++ Build Tools needed). It checks every Cargo target, runs tests, then runs player smoke and editor model QA on all three scenes. BUILD_WINDOWS.bat gates release packaging on these checks. The GitHub workflow can run on a Windows runner after the project is uploaded to a repository; it has not been run here.

## Model QA (headless, implemented; not executed here)
- Load each sector; edit player transform; undo and redo.
- Instantiate prefab with unique IDs; reparent; reject cycle.
- Paint and erase a tile.
- Verify UV bounds for player animation clips.
- Save/reload a temporary scene; compare values.
- Run one simulation step in Play copy; verify source scene unchanged.
- Smoke physics, input movement, script errors, objective completion; unit tests for damage and exit gate.

## Manual editor QA (not executed)
| Test | Expected outcome |
|---|---|
| Project Manager > open examples/signal_lost/project.forge.json | Scene opens, graphics visible |
| Open level2.forge.json and level3.forge.json individually | Different backgrounds and platform placements |
| Drag sprite from browser into scene | New sprite at cursor, undo removes it |
| Move/rotate/scale player | Inspector matches viewport; undo/redo restores state |
| Reparent child and move parent | Child follows parent |
| Animation tab > load Run | Timeline preview changes frames |
| Tilemap paint/erase | Visible tile with correct collision setting |
| Lua and graph editor | No new errors on demo play |
| Play > collect core, touch enemy, reach exit | HUD reports score and health; source scene stays unchanged after Stop |
| Camera Preview, docking, theme switch | Correct rendering and panel state |
| Save As to another folder | Sprite, sound, particle and normal map files copied if referenced |
| Close/reopen editor | Workspace and recovery behave as configured |
| Launch packaged Signal Lost | Menu, three sectors, audio, victory/retry |

Full GUI interaction, audio playback, GPU rendering and Windows packaging are not verified until this checklist is run on a real Windows system. No APK or iOS build.
