"""Static regression checks, not execution of Rust or GPU code."""
import unittest, re, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_group_wiring(self):
  s=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text(); core=(ROOT/'src/ultimate.rs').read_text()
  for t in ['Select all keys','Copy group','Paste at playhead','Delete group','Group Bezier','v.modifiers.ctrl','paste_scalar_group']:
   self.assertIn(t,s)
  for t in ['keys.len()+clip.len()>1024','keys.iter().chain(pending.iter())','keys.extend(pending)','paste_relative_and_atomic']:
   self.assertIn(t,core)
 def test_project_import_and_dependencies(self):
  s=(ROOT/'src/asset_pipeline.rs').read_text(); db=(ROOT/'src/max_tools.rs').read_text()
  for t in ['project_image_index','cached_hashes','256*1024*1024','bytes==raw','reuse_across_imports','ty.is_file()','remove_dir(&dest)']:
   self.assertIn(t,s)
  for t in ['o.pro.sound.iter()','read_composite_for_edit','32768','Unsafe or oversized asset database']:
   self.assertIn(t,db)
 def test_graph_and_light_quality(self):
  s=(ROOT/'src/bin/forge_editor/ultimate.rs').read_text(); g=(ROOT/'src/gpu_scene.rs').read_text(); settings=(ROOT/'src/bin/forge_editor/gameplay_ui.rs').read_text()
  for t in ['Auto save/load layout','response.drag_stopped()','layout_released','signature30','old==bytes']:
   self.assertIn(t,s)
  self.assertIn('Vec2::splat(p.settings.shadow_softness.clamp(0.,32.))',g)
  for t in ['("Low",1,0.)','("Balanced",2,6.)','("High",3,16.)']:
   self.assertIn(t,settings)
 def test_metrics_and_lock(self):
  s=(ROOT/'src/bin/forge_editor/max.rs').read_text();self.assertIn('"lighting_sync_cpu_ns":sync_ns',s);self.assertIn('"gpu_frame_ms":null',s)
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.43');pat=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1)
  raw='[[package]]\nname = "forge_engine"\nversion = "0.7.29"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';result,n=re.subn(pat,r'\g<1>0.7.43\g<2>',raw);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',result)
if __name__=='__main__':unittest.main()
