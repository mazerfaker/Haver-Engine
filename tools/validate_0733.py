"""Static wiring and independent reference checks, NOT Rust execution."""
import unittest,json,re,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_overshoot_and_lanes(self):
  cubic=lambda t,a,b:3*(1-t)**2*t*a+3*(1-t)*t*t*b+t**3
  self.assertGreater(cubic(.8,.4,2),1)
  self.assertEqual(cubic(1,-1,2),1)
  scene=json.loads((ROOT/'examples/timeline_overshoot/project.forge.json').read_text())
  self.assertTrue(any('clip_keys' not in o['motion'] and any(k.get('bezier',[0,0])[1]>1 for ch in o['motion']['property_channels'].values() for axis in ch for k in axis if k.get('bezier')) for o in scene['objects'] if 'property_channels' in o.get('motion',{})))
  ui=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text();self.assertIn('multi_lane_canvas(ui,o',ui);self.assertIn('response.drag_stopped()',ui);self.assertIn('move_multi_channel(&mut o.motion',ui)
  self.assertIn('sample_clip_with_base', (ROOT/'src/runtime.rs').read_text())
 def test_bus_dsp(self):
  p=json.loads((ROOT/'examples/bus_dsp/project.forge.json').read_text());self.assertEqual(p['settings']['audio_buses']['Wet room']['effects']['reverb'],.7)
  self.assertTrue(any(s.get('custom_bus')=='Wet room' and 'effects' not in s for o in p['objects'] if (s:=o.get('pro',{}).get('sound'))))
  player=(ROOT/'src/bin/forge_player.rs').read_text();self.assertIn('s.effective_effects(&game.project.settings)',player);self.assertIn('sound.effective_effects(&game.project.settings)',player)
 def test_cache_and_ranking(self):
  s=(ROOT/'src/max_tools.rs').read_text();self.assertIn('if !use_cache{old.clear();}',s);self.assertIn('entry.present||!entry.references.is_empty()',s)
  a=[(4,0),(1,1),(1,2),(3,3),(2,4),(0,5)];best=[]
  for item in a:
   i=next((i for i,v in enumerate(best) if v[0]>item[0]),len(best));best.insert(i,item);best=best[:4]
  self.assertEqual(best,sorted(a,key=lambda x:x[0])[:4]);self.assertIn('best_n_fixed::<_,4>',(ROOT/'src/gpu_scene.rs').read_text())
 def test_lock_and_prefab(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.43');pat=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1)
  raw='[[package]]\nname = "forge_engine"\nversion = "0.7.32"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(pat,r'\g<1>0.7.43\g<2>',raw);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',out)
  ui=(ROOT/'src/bin/forge_editor/ultimate.rs').read_text();self.assertIn('Local object hierarchy',ui);self.assertIn('Arc::make_mut(doc)',ui);self.assertIn('update_composite_prefab',ui)
if __name__=='__main__':unittest.main()
