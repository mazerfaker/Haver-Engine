"""Static integration gates; Rust DSP tests are executed by cargo test."""
import unittest,tomllib,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def read(p):return (ROOT/p).read_text(encoding='utf-8')
class Ultimate43(unittest.TestCase):
 def test_background_index_and_transaction_guard(self):
  p=read('src/bin/forge_editor/pro.rs');m=read('src/bin/forge_editor/max.rs');t=read('src/bin/forge_editor/master.rs')
  for term in ['haver-asset-scan','h.is_finished()','e.pro.last_scan==Some(stamp)','*visited>=32768']:self.assertIn(term,p)
  for term in ['haver-dependency-index','e.project==snapshot','database_busy']:self.assertIn(term,m)
  self.assertIn('super::max::database_busy(e)',t)
 def test_controls_and_dynamics(self):
  a=read('src/bin/forge_editor/animation_studio.rs');d=read('src/audio_effects.rs');u=read('src/bin/forge_editor/gameplay_ui.rs')
  for term in ['Snap playhead to frame','Visible range','frame_sample_time']:self.assertIn(term,a)
  for term in ['process_bus_dynamics','gate_and_stereo_link','compresses_and_reports']:self.assertIn(term,d)
  self.assertIn('Process PCM16 WAV and export',u)
  self.assertIn('prefab_picker43',read('src/bin/forge_editor/master.rs'))
 def test_light_lab_and_release(self):
  shader=read('src/gpu_scene.wgsl');self.assertLess(shader.index('if !reaches0'),shader.index('textureSampleLevel(normals'))
  self.assertIn('pass.draw(0..6,0..N)',read('experimental/gpu_instancing_lab.rs'))
  for f in ['forge_particle_compute.wgsl','forge_instanced_particles.wgsl']:self.assertTrue((ROOT/'assets/shaders'/f).is_file())
  self.assertIn('cargo run --locked --release',read('tools/run_gpu_instancing_lab.ps1'))
  w=read('.github/workflows/windows.yml');self.assertLess(w.index('Modular feature checks'),w.index('Cargo check, test, build --release'))
  self.assertEqual(tomllib.loads(read('Cargo.toml'))['package']['version'],'0.7.44')
  lock=tomllib.loads(read('Cargo.lock'));self.assertEqual([p['version'] for p in lock['package'] if p['name']=='forge_engine'],['0.7.44'])
if __name__=='__main__':unittest.main()
