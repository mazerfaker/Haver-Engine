# Forge Engine v0.7.21 — Studio Refinement & Runtime Safety

Base: exact published v0.7.20 source archive. This is a targeted refinement, not a verified Windows release.

## Source changes
- Atlas Canvas: open Tools / Animation on an object with an irregular atlas and an already loaded texture. Select a frame in the active clip (zero-based), drag the selected rectangle, resize with the bottom-right handle, snap its bounds to actual image pixels, then Store clip. Limits maintain positive dimensions and normalized UVs. The frame controls now use actual region count rather than grid dimensions; the frame strip samples inside the selected frame to avoid floating-point boundary errors. Transform timeline duration accounts for per-frame timing. No automatic animator or Auto Rig.
- Recursive docking: dropping a tab onto a tab in the same group reorders it before the target instead of always appending. Previous split, persistence and legacy workspace remain intact. Still limited to four main panels; play mode remains legacy.
- Player scene audio: scene Sound components start once on initial load, campaign level transition or explicit restart. Old component voices are removed on transition (including loops); one-shot voices are not recreated every frame after completion. Mixer, gain smoothing, stereo pan and offline PCM16 effects remain intact. Standalone demo cue sounds are not part of this voice lifecycle; dynamic spawn/despawn audio and live DSP remain out of scope.
- Linked prefab refresh: refuses changed parent topology or mixed library sources before writing any object. Gameplay override no longer copies the entire ProComponents struct (which incorrectly kept old physics and graph positions); scripts override preserves graph layout. Appearance override keeps pivot. Nonempty sound asset paths validated. Existing behavior preserving local transform, identity and hierarchy remains. Structural merge and automatic live refresh not implemented.
- Build: QUICK_TEST.bat executes all-targets Rust unit tests including GPU examples with a separate log and preserves the existing build. Cargo.lock migration accepts 0.7.20, changes root version only; Windows CI artifact name corrected. No dependency or profile change.
- Six new Rust tests: atlas boundary, pixel snapping, variable frame sampling, prefab atomic hierarchy rejection, isolated gameplay overrides, dock tab reorder. These are provided, not run here.

## What remains
No changes to GPU shaders or particle budget; no hardware instancing or live DSP, no complete redesign, no arbitrary OS docking, no prefab topology merging. No CPU/GPU/RAM/VRAM benchmark claims. No compiler present in this environment. Last confirmed full build v0.7.15; v0.7.16 GPU scene/lab confirmed by user; later versions unverified.

## Test order on Windows
1. Back up working files and extract update into v0.7.20 root. Keep Cargo.lock and target; lock root version migrated on first check.
2. QUICK_CHECK.bat, QUICK_TEST.bat, QUICK_BUILD.bat, RUN_EDITOR_DEBUG.bat. Send forge-quick-check.log or forge-quick-test.log if any failure. Do not trust a stale debug EXE if build fails.
3. Open examples/atlas_regions/project.forge.json, select actor, Tools / Animation, load clip, open Atlas canvas; drag, resize, snap, Store clip, save/reload. RUN_ATLAS_SCENE.bat for visual regression.
4. Enable Studio / Recursive docking workspace, put two panels into same tab leaf, drop one onto the other, restart editor; check order persistence.
5. Run Signal Lost campaign, restart or move to next level; verify sounds do not accumulate and scene-specific loop is replaced. Verify linked prefab refresh with unchanged hierarchy, then change source parent and confirm refusal; test independent Gameplay and Physics overrides.
6. RUN_GPU_SCENE.bat and RUN_GPU_QUADS_SCENE.bat for prior shader paths; CHECK_WINDOWS.bat and BUILD_WINDOWS.bat after quick checks.
