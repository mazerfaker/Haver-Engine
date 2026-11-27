# Haver Engine v0.7.47 — Audio Mixer & Lighting
Base: CI-verified 9a10d2387e9a4f9b9ca3324c737b3b8bea61df98. Candidate pending CI.
- Extract reusable stereo-linked dynamics gain envelope, retaining offline PCM16 algorithm and output quantization. No live audio integration yet.
- Three presets and reset; analyze WAV without exporting; reject existing canonical source path as output.
- Lighting shader skips zero-intensity lights and fully blocked contributions. No new shader uniforms or shadow sampling changes.
- Two new Rust regression tests for envelope attack/release/reset, presets and finite bounded gain. Static shader wiring checks; actual GPU output and performance not tested by CI.
- Existing audio paths, asset tools and animation retained. No new dependencies.
