"""Static checks for v0.7.44; do not represent Rust compilation or GPU/audio playback."""
import json,re,tomllib,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_property_fixture(self):
  root=ROOT/'examples/property_channels'; p=json.loads((root/'project.forge.json').read_text()); o=next(x for x in p['objects'] if x.get('motion',{}).get('property_channels'));self.assertTrue((root/o['sprite']).is_file());self.assertEqual(set(o['motion']['property_channels']),set(o['motion']['clip_keys']));self.assertFalse(o['player']);self.assertEqual(o['body'],'None')
  for tracks in o['motion']['property_channels'].values():
   self.assertEqual(len(tracks),3);self.assertNotEqual([k['time'] for k in tracks[0]],[k['time'] for k in tracks[1]]);self.assertEqual(tracks[2],[])
  s=(ROOT/'src/prefab_studio.rs').read_text();self.assertEqual(s.count('property_channels.values_mut()'),3);self.assertIn('next.motion.property_channels=old.motion.property_channels.clone()',s)
 def test_dsp_control_wiring(self):
  decoder=(ROOT/'src/bin/forge_player/audio_dsp.rs').read_text();self.assertIn('Arc<AtomicU64>',decoder);self.assertIn('self.until_poll=128',decoder);self.assertIn('self.dsp.set_effects',decoder)
  player=(ROOT/'src/bin/forge_player.rs').read_text();self.assertIn('egui::Key::F8',player);self.assertIn('c.set(sound.effective_effects(&game.project.settings).unwrap_or_default())',player);self.assertIn('let settings=if s.looping{PlaybackSettings::ONCE',player)
  editor=(ROOT/'src/bin/forge_editor/audio_preview.rs').read_text();self.assertIn('PlaybackSettings::ONCE.with_volume',editor);self.assertIn('Apply effects to selected sound',editor)
 def test_lighting_and_atlas(self):
  w=(ROOT/'src/gpu_scene.wgsl').read_text();self.assertLess(w.index('if dist2>=light.z*light.z'),w.index('if !any_blocker'));self.assertIn('visible=f32(samples)',w);self.assertIn('1.0-dist/light.z',w)
  s=(ROOT/'src/atlas_studio.rs').read_text();self.assertIn('old==data',s);self.assertIn('fs::rename(&temp,&target)',s)
  e=(ROOT/'src/bin/forge_editor.rs').read_text();self.assertIn('Scene saved; {errors} atlas sync errors',e)
 def test_version_and_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.44');s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.26"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"\n';out,n=re.subn(pat,r'\g<1>0.7.44\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.44"',out);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
