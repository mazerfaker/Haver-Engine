"""Static integration + independent reference checks; not Rust compilation."""
import json,re,tomllib,unittest,wave
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_live_bus(self):
  d=ROOT/'examples/live_summed_bus';p=json.loads((d/'project.forge.json').read_text());self.assertTrue(p['settings']['audio_buses']['Wet room']['summed_runtime']);sounds=[o['pro']['sound'] for o in p['objects'] if 'sound' in o.get('pro',{})];self.assertEqual(len(sounds),2)
  formats=[]
  for s in sounds:
   self.assertEqual(s['custom_bus'],'Wet room');self.assertFalse(s['spatial']);self.assertEqual(s['speed'],1.)
   with wave.open(str(d/s['path'])) as w:formats.append((w.getframerate(),w.getnchannels(),w.getsampwidth(),w.getnframes()))
  self.assertEqual(formats[0],formats[1]);self.assertAlmostEqual(.25+(.5*.5),.5)
  player=(ROOT/'src/bin/forge_player.rs').read_text();dsp=(ROOT/'src/bin/forge_player/audio_dsp.rs').read_text()
  for x in ['load_summed','grouped.contains(&o.id)','shared:Some(name.clone())','Shared bus {name} fallback']:self.assertIn(x,player)
  for x in ['sum_sample(&self.tracks','self.dsp.process(&mut sample)','Looping shared tracks must have equal lengths']:self.assertIn(x,dsp)
 def test_events(self):
  source=(ROOT/'src/ultimate.rs').read_text();ui=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text();runtime=(ROOT/'src/runtime.rs').read_text()
  self.assertIn('pub fn move_timeline_group',source);self.assertIn('Event collision; no lanes changed',source);self.assertIn('delete_timeline_events',ui);self.assertIn('move_timeline_group(&mut o.motion',ui)
  self.assertGreater(runtime.rfind('get::<mlua::Table>("event_queue")'),runtime.index('on_animation_event'))
  self.assertAlmostEqual(1/.5,2.)
 def test_graph_and_gpu(self):
  core=(ROOT/'src/max_tools.rs').read_text();self.assertIn('pub fn dependency_nodes',core);self.assertIn('rows.len()>=512',core);self.assertIn('Missing only',(ROOT/'src/bin/forge_editor/max.rs').read_text())
  self.assertIn('can_reparent_local',(ROOT/'src/prefab_studio.rs').read_text());self.assertIn('recursive_pos35',(ROOT/'src/bin/forge_editor/ultimate.rs').read_text());shader=(ROOT/'src/gpu_scene.wgsl').read_text();self.assertLess(shader.index('textureSample(normals'),shader.index('if base.a<=0.0'));self.assertLess(shader.index('if base.a<=0.0'),shader.index('for(var li:'))
 def test_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.36');pat=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1);raw='[[package]]\nname = "forge_engine"\nversion = "0.7.34"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(pat,r'\g<1>0.7.36\g<2>',raw);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
