"""Static fixtures and source wiring; not a compiler or execution test."""
import json, unittest, tomllib, re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_visual_fixture(self):
  p=ROOT/'examples/visual_channels'; d=json.loads((p/'project.forge.json').read_text()); o=next(o for o in d['objects'] if o.get('motion',{}).get('visual_channels')); self.assertTrue((p/o['sprite']).is_file()); self.assertEqual(set(o['motion']['visual_channels']),set(o['motion']['property_channels']));
  for clip in o['motion']['visual_channels'].values():
   self.assertEqual(len(clip),3); self.assertTrue(all(0.01<=k['value']<=16 for c in clip[:2] for k in c)); self.assertTrue(all(0<=k['value']<=1 for k in clip[2]));
  for f in ['src/bin/forge_player.rs','src/bin/forge_editor.rs','src/bin/forge_editor/viewport.rs','src/gpu_scene.rs']: self.assertIn('sprite_visual_alpha',(ROOT/f).read_text())
 def test_wiring(self):
  for f,terms in {'src/audio_effects.rs':['self.target','self.smoothing','DSP_PRESETS'],'src/bin/forge_player.rs':['settings.voice','settings.ambience','s.gain(&game.project.settings)'],'src/gpu_scene.rs':['solid_cache.query_into','solid_grid.insert_bounded'],'src/bin/forge_editor/ultimate.rs':['prefab_dependency_rows','layout28'],'src/bin/forge_editor/animation_studio.rs':['trim_all_frames','Enable visual channels']}.items():
   for term in terms:self.assertIn(term,(ROOT/f).read_text())
 def test_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.48');s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1)
  for v in range(14,30):
   text=f'[[package]]\nname = "forge_engine"\nversion = "0.7.{v}"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"\n';out,n=re.subn(pat,r'\g<1>0.7.48\g<2>',text);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
