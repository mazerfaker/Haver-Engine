"""Compare two benchmark.json files; positive speedup means faster."""
import json,sys
if len(sys.argv)!=3: raise SystemExit('Usage: python tools/compare_benchmarks.py baseline.json new.json')
a,b=(json.load(open(x,encoding='utf-8-sig')) for x in sys.argv[1:]);
if (a['objects'],a['frames'])!=(b['objects'],b['frames']): raise SystemExit('Incompatible benchmark workloads')
for k in ('load_ms','simulation_mean_ms','simulation_p95_ms','simulation_max_ms'):
 x,y=a[k],b[k]; print(f'{k}: {x:.4f} -> {y:.4f} ms ({(1-y/x)*100:+.2f}% reduction)' if x>0 else f'{k}: {x} -> {y}')
