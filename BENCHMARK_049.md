# Experimental benchmark capture

Run from the repository root in PowerShell:

```powershell
$env:HAVER_GPU_INSTANCES='10000'
$env:HAVER_BENCHMARK_JSON='benchmark.json'
cargo run --release --locked --no-default-features --features gpu-instancing-lab --example gpu_instancing_lab
```

Repeat with 1000, 5000, 10000, 50000, 100000. The process exits after 120 warmup and 600 valid frame intervals. Results are CPU wall-clock intervals, not GPU timestamps; pipeline readiness and actual drawing are not certified by the report. No VRAM or GPU timing is inferred. This is not yet a comparison against the standard renderer.
