"""Static integration checks for audio and lighting."""
import unittest,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def read(p):return (ROOT/p).read_text(encoding='utf-8')
class AudioLight47(unittest.TestCase):
 def test_dynamics(self):
  a=read('src/audio_effects.rs');u=read('src/bin/forge_editor/gameplay_ui.rs')
  for term in ['pub struct DynamicsEnvelope','envelope.next_gain(peak)','envelope_attack_release_and_reset','presets_valid_and_gain_bounded']:self.assertIn(term,a)
  self.assertIn('forge_engine::DYNAMICS_PRESETS',u)
  self.assertIn('Analyze PCM16 WAV (no export)',u)
  self.assertIn('if !analyze',u)
 def test_lighting_and_version(self):
  s=read('src/gpu_scene.wgsl')
  self.assertLess(s.index('light.w==0.0'),s.index('let any_blocker'))
  self.assertLess(s.index('if visible==0.0'),s.index('visible/=f32(samples)'))
  v=tomllib.loads(read('Cargo.toml'))['package']['version']
  self.assertEqual(v,'0.7.47')
  self.assertEqual([p['version'] for p in tomllib.loads(read('Cargo.lock'))['package'] if p['name']=='forge_engine'],[v])
if __name__=='__main__':unittest.main()
