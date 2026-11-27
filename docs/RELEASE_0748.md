# Haver Engine v0.7.48 — Performance & GPU
Base: CI-verified 291b939b605677d26ff99c501d8209efce70de9d. Candidate pending CI.
- Isolated GPU instancing lab configurable from 1 to 65536 instances; default 1024. Compute and draw counts share the same bounded buffer count. Frame diagnostics and optional GPU timestamps, per-workload log files.
- Manual hardware commands: `powershell -ExecutionPolicy Bypass -File tools/run_gpu_instancing_lab.ps1 -Instances 1024`, repeat for 8192 and 65536; optional `-GpuProfile`.
- Discard completed texture decode if its path left the active scene or file timestamp changed; expose discard count. Existing texture-cache budget and behavior retained.
- UI frame sample mean, estimated FPS, P99 and reset; JSON includes summary. UI timing is not GPU timing, payload estimates are not actual VRAM.
- Tests for frame statistics, cache freshness and lab count bounds. CI compiles the experimental lab; actual GPU execution, visual output and performance improvements remain unverified. No integration of hardware instancing into the production renderer, no new dependencies.
