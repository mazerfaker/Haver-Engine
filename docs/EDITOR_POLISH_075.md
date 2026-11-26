# Editor Polish 0.7.5

Manual regression checklist (not executed):
- Resize editor to 1024x768; toolbar wraps, inspector scrolls.
- Open Signal Lost and press Home; scene centered and zoomed within allowed range 20–400%.
- Select an axis, then click a different sprite; free XY movement restored.
- Delete selected object then undo/redo; missing selection IDs removed.
- Play and attempt hierarchy root drop; original scene must remain unchanged. Stop and compare scene.
- Open several projects; most recent appears first.
- Check UA/EN help and Escape deselection.
- Run CHECK_WINDOWS.bat then BUILD_WINDOWS.bat.

Known limitations: no new docking library or full localization; camera fit can be limited by existing zoom bounds. Rust tests written, not executed here.
