# Haver Engine v0.7.43 — Ultimate Tools & Performance
Base: CI-verified b8170ec1f6f700e989b4686d7f69c3a00c5bb5bd. Candidate pending own CI.

- Editor: updated dark palette, type hierarchy, header and save state; legacy panels/layouts remain.
- Timeline: first/previous/next/last frame, frame snap, all-lane and visible-range selection; existing curve tools remain. No Auto Rig or automatic animator.
- Assets: bounded background image scan; dependency index worker with scene-snapshot check; file transaction blocked while the database worker is active. Cache retained.
- Audio: optional offline PCM16 compressor and gate, stereo-linked attenuation, peak/RMS/reduction report; three DSP tests. Existing live mixer unchanged; live dynamics deferred.
- Prefabs: snapshot source/destination object pickers for existing journalled, fingerprint-checked transfer. Used/scripted prefabs remain rejected; automatic dependent-prefab migration deferred.
- Lighting: early radius rejection before normal sample. GPU output and speed not verified by compiler; soft-shadow algorithm unchanged.
- GPU Instancing: manual hardware runner with logs; standalone experiment remains isolated. No actual GPU runtime result in CI.
- Performance: bounded scan and reduced UI blocking; no claimed FPS/VRAM improvements until benchmarks.
- CI: run feature checks before expensive release build, verify staged editor QA and GPU shader files; retain cache, full source ZIP and lock-preserving overlay against exact base. No third-party Rust dependency changes.
