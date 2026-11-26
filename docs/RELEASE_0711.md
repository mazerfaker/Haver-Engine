# v0.7.11 — Combat & UI Refinement

Based on v0.7.10. Source-only, not a verified executable. No Auto Rig.

- Directional melee range, heavy enemies (4 HP), boss chase phases (6 HP; phase 2 at 3 HP), shared attack clip in editor and player. Original hero atlas retained, extended atlas adds four attack frames. AABB range, not frame-specific collision shapes.
- Adaptive HUD buttons scale with design resolution and stay within screen boundaries. Inventory/container editor not implemented.
- Standalone pause-menu mixer adjusts gain on tagged persistent music/SFX sinks. One-shot cues retain volume at spawn. No audio audition or DSP effects.
- Dock zone drag strip adds Float target; prior panels retained. Not arbitrary split/tab docking.
- Theme settings are applied only on change instead of every UI frame. No measured FPS or binary size improvements.
- Build workflow and portable filename updated to 0.7.11 (previously stale 0.7.9). GPU shader prototypes unchanged; no compute dispatch or normal-map pipeline integration.

## Test status
Static demo asset checks for four scenes, JSON/TOML parsing, two Python packaging tests passed. New Rust unit tests for directional attack and adaptive UI were authored, not executed. Rust compiler not available in this environment. Run CHECK_WINDOWS.bat and QA_MATRIX.bat on Windows. Build with BUILD_WINDOWS.bat. Retain a separate copy of v0.7.10.
