"""Static fixture and source checks; not Rust or audio/GPU execution."""
import unittest,json,tomllib,re,wave
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_routing_fixture(self):
  p=ROOT/'examples/audio_routing';d=json.loads((p/'project.forge.json').read_text());b=d['settings']['audio_buses'];sounds=[o['pro']['sound'] for o in d['objects'] if o.get('pro',{}).get('sound')];self.assertEqual(b['Wet room']['parent'],'Environment');self.assertAlmostEqual(b['Wet room']['gain']*b['Environment']['gain'],.6);self.assertEqual(b['Environment']['parent'],'Ambience');self.assertGreater(sounds[1]['effects']['reverb'],0)
  for s in sounds:
   self.assertIn(s['custom_bus'],b)
   with wave.open(str(p/s['path'])) as w:self.assertEqual(w.getsampwidth(),2)
 def test_audio_source_wiring(self):
  for f,terms in {'src/pro_gameplay.rs':['custom_bus_gain','valid_audio_buses','0..=32'],'src/audio_effects.rs':['self.reverb_target','self.comb_pos','self.damp'],'src/bin/forge_player/audio_dsp.rs':['q(fx.reverb,0.,1.)<<48','(v>>48)&65535'],'src/bin/forge_player.rs':['Custom buses','s.gain(&game.project.settings)'],'src/bin/forge_editor/audio_preview.rs':['request.2.reverb']}.items():
   for term in terms:self.assertIn(term,(ROOT/f).read_text())
 def test_asset_graph_and_cache_wiring(self):
  for f,terms in {'src/asset_pipeline.rs':['import_image_batch','import_hash(&raw)','b==raw','reader.limits(limits)','remove_dir_all(dest)'],'src/gpu_scene.rs':['light_signature.iter().copied().eq','solid_signature.iter().copied().eq','std::mem::take(&mut self.state.scratch)','last_index_cpu_ns'],'src/bin/forge_editor/ultimate.rs':['Save graph layout','Load graph layout','Open selected dependency (discard current draft)'],'src/bin/forge_editor/animation_studio.rs':['Shift entire channel','bezier29'],'src/bin/forge_editor/max.rs':['Export performance snapshot','"gpu_frame_ms":null','"total_vram_bytes":null']}.items():
   for term in terms:self.assertIn(term,(ROOT/f).read_text())
 def test_version_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.44');s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1)
  for version in range(14,30):
   old=f'[[package]]\nname = "forge_engine"\nversion = "0.7.{version}"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(pat,r'\g<1>0.7.44\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.44"',out);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
