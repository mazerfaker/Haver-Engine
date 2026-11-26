# 0.7.10 — partial refinement

Implemented: shared melee attack (J), cooldown, enemy HP, boss gate, fourth arena and pixel boss asset; campaign packaging and HUD widget size controls. Original levels unchanged.

Not implemented: GPU compute/render graph integration, full docking, audio effects or preview, inventory system. Existing CPU renderers and editor preserved. No Auto Rig.

Build: use CHECK_WINDOWS.bat then BUILD_WINDOWS.bat. Rust compiler unavailable in authoring environment. No EXE or runtime test claim. Smoke objective bypasses boss to isolate collectible test; separate Rust unit test checks boss damage.
