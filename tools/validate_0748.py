"""Static integration gates for performance and experimental GPU workload."""
import unittest,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def read(p):return (ROOT/p).read_text(encoding='utf-8')
class Perf48(unittest.TestCase):
 def test_lab(self):
  s=read('experimental/gpu_instancing_lab.rs')
  for term in ['MAX_INSTANCES:u32=65536','b.count.div_ceil(64)','pass.draw(0..6,0..b.count)','FrameTimeDiagnosticsPlugin','workload_bounds']:self.assertIn(term,s)
  self.assertIn('ValidateRange(1,65536)',read('tools/run_gpu_instancing_lab.ps1'))
 def test_cache_and_stats(self):
  v=read('src/bin/forge_editor/viewport.rs');m=read('src/bin/forge_editor/max.rs')
  for term in ['decode_is_current(active,stamp,actual)','cache.poll(&mut images,&paths)','stale_decode_rejected']:self.assertIn(term,v)
  self.assertIn('ui_frame_summary',m)
  self.assertIn('stale_texture_decodes',m)
  self.assertIn('pub fn frame_summary',read('src/performance.rs'))
 def test_version(self):
  version=tomllib.loads(read('Cargo.toml'))['package']['version']
  self.assertEqual(version,'0.7.48')
  self.assertEqual([p['version'] for p in tomllib.loads(read('Cargo.lock'))['package'] if p['name']=='forge_engine'],[version])
if __name__=='__main__':unittest.main()
