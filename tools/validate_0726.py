"""Static fixtures and wiring checks; no Rust execution implied."""
import json, re, unittest, wave, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Checks(unittest.TestCase):
 def test_audio_fixture(self):
  root=ROOT/'examples/audio_runtime';p=json.loads((root/'project.forge.json').read_text());sounds=[o['pro']['sound'] for o in p['objects'] if 'sound' in o.get('pro',{})];self.assertEqual(len(sounds),2);self.assertEqual(sounds[0]['path'],sounds[1]['path']);self.assertNotIn('effects',sounds[0]);self.assertEqual(sounds[1]['effects']['room'],.85)
  with wave.open(str(root/sounds[0]['path'])) as f:self.assertEqual((f.getnchannels(),f.getsampwidth(),f.getframerate(),f.getnframes()),(1,2,16000,32000))
 def test_audio_wiring_and_fallback(self):
  s=(ROOT/'src/bin/forge_player.rs').read_text();self.assertIn('add_audio_source::<audio_dsp::DspAudio>()',s);self.assertIn('DSP fallback',s);self.assertIn('assets.load::<AudioSource>',s);self.assertIn('dsp_count<16',s);self.assertIn('dsp_budget.saturating_sub(bytes)',s)
  d=(ROOT/'src/bin/forge_player/audio_dsp.rs').read_text();self.assertIn('self.dsp.process(&mut sample)',d);self.assertIn('cap as u64+1',d);self.assertIn('tail_total',d)
 def test_metadata_composer_wiring(self):
  a=(ROOT/'src/atlas_studio.rs').read_text();self.assertIn('atlas-state-{}.json',a);self.assertIn('frame_geometry',a);self.assertIn('fs::rename(&temp,&target)',a)
  p=(ROOT/'src/prefab_studio.rs').read_text();self.assertIn('load_library_prefab_with_override(&root,file,Some((&path,&bytes)))',p);self.assertIn('current.original!=doc.original',p);self.assertIn('write(&backup,&current.original)',p)
  ui=(ROOT/'src/bin/forge_editor/ultimate.rs').read_text();self.assertIn('Save changes + backup',ui);self.assertIn('Open composition',ui)
 def test_shader_and_lock(self):
  w=(ROOT/'src/gpu_scene.wgsl').read_text();self.assertIn('select(1u,5u',w);self.assertIn('vec2<f32>(-1e20)',w);self.assertIn('hit=true;break;',w)
  p=tomllib.loads((ROOT/'Cargo.toml').read_text());self.assertEqual(p['package']['version'],'0.7.42');s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1);self.assertRegex('[[package]]\nname = "forge_engine"\nversion = "0.7.25"',pat)
if __name__=='__main__':unittest.main()
