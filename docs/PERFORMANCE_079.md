# Forge Engine 0.7.9 — Engine Refinement

Based on 0.7.8. No Auto Rig. CPU lighting/particles and existing game data retained.

Implemented (Rust source, not compiled): viewport cache evicts least recently used non-visible textures when admitting a new texture; visible paths pinned, failures retried when visible set changes. The cache counts decoded image estimates, not total RAM/VRAM. Budget pressure may leave placeholders; worker queues and other Bevy/egui caches are outside the limit. Added conservative static tile grouping API; it is not connected to draw submission and does not reduce draw calls yet.

Implemented and executed (Python): asset ZIP packaging, optional lossless PNG recompression, SHA-256 manifest, conservative reference scan and opt-in pruning. Pruning automatically disabled for projects containing scripts; other implicit references require manual review. Source files remain unchanged. Demo result: 341900 -> 340396 bytes before ZIP compression (1504 bytes saved), 18 files, no excluded assets. Two unit tests passed; three demo scene validations passed.

GPU lighting and particle WGSL prototypes from 0.7.8 are preserved. No new Bevy GPU pipeline, compute dispatch or GPU particle rendering is integrated. Global RAM/VRAM cache and actual chunk batching are not complete.

QA_MATRIX.bat checks independent Cargo feature sets, full and minimal tests on Windows; wired to GitHub Actions. Rust/cargo unavailable in current environment, so no Rust compilation, WGSL validation, Windows EXE or FPS claims. Previous source archive remains unchanged.

Usage: python tools/pack_assets.py examples/signal_lost output.zip --optimize-png (Pillow required only for PNG optimization). Use --prune only after auditing implicit asset references. Extract archive for use with Player; no executable included. Run QA_MATRIX.bat then BUILD_WINDOWS.bat on Windows.
