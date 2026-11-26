"""Static checks only. Not Rust compilation or GPU/audio execution."""
from pathlib import Path
import json, unittest
ROOT=Path(__file__).resolve().parents[1]
class NewChecks(unittest.TestCase):
 def test_channel_fixture(self):
  root=ROOT/'examples/animation_channels'; p=json.loads((root/'project.forge.json').read_text()); o=next(o for o in p['objects'] if 'clip_keys' in o.get('motion',{})); self.assertFalse(o['player']); self.assertEqual(o['body'],'None'); self.assertEqual(o['motion']['transitions'][0]['to'],'Loop'); self.assertEqual(set(o['motion']['clip_keys']),set(o['clips'])); self.assertTrue((root/o['sprite']).exists()); self.assertFalse(o['clips']['Once']['looping']);self.assertTrue(o['clips']['Loop']['looping']);self.assertIn('sample_clip', (ROOT/'src/runtime.rs').read_text())
 def test_instance_lab(self):
  rs=(ROOT/'experimental/gpu_instancing_lab.rs').read_text(); wgsl=(ROOT/'assets/shaders/forge_instanced_particles.wgsl').read_text(); self.assertIn('pass.draw(0..6,0..N)',rs);self.assertIn('@builtin(instance_index)',wgsl);self.assertIn('particles[i]',wgsl);self.assertNotIn('for(',wgsl);self.assertIn('buffers:vec![]',rs); self.assertIn('gpu-instancing-lab', (ROOT/'GPU_INSTANCING_LAB.bat').read_text())
 def test_stream_and_composer(self):
  self.assertIn('self.dsp.process(&mut out)',(ROOT/'experimental/audio_dsp_lab.rs').read_text());self.assertIn('create_new(true)',(ROOT/'src/prefab_studio.rs').read_text());self.assertIn('create_composite_prefab',(ROOT/'src/bin/forge_editor/ultimate.rs').read_text()); self.assertIn('clip_keys.values_mut().flatten()', (ROOT/'src/prefab_studio.rs').read_text())
if __name__=='__main__':unittest.main()
