"""Static source/fixture checks only, not a Rust runtime test."""
import unittest,json,wave,struct,tomllib,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def read(p):return (ROOT/p).read_text()
class Changes38(unittest.TestCase):
 def test_mixed_pcm_fixture(self):
  d=ROOT/'examples/mixed_pcm';p=json.loads((d/'project.forge.json').read_text());self.assertTrue(p['settings']['audio_buses']['Environment']['cascade_runtime']);formats=[]
  for o in p['objects']:
   s=o.get('pro',{}).get('sound')
   if s:
    with wave.open(str(d/s['path'])) as w:formats.append((w.getsampwidth(),w.getframerate(),w.getnchannels(),w.getnframes()))
  self.assertEqual({f[0] for f in formats},{1,3});self.assertEqual(formats[0][1:],formats[1][1:]);self.assertEqual((192-128)*256,16384);self.assertEqual(int.from_bytes(bytes([0,64]),'little',signed=True),16384)
  code=read('src/bin/forge_player/audio_dsp.rs');self.assertIn('normalize_pcm(&raw,cap)',code);self.assertIn('Normalized PCM exceeds budget',code);self.assertIn('fn mixed_formats',code)
 def test_onion_skin_and_navigation(self):
  s=read('src/bin/forge_editor/animation_studio.rs');self.assertIn('Onion Skin',s);self.assertIn('preview_frame_time',s);self.assertIn('if onion&&n>1',s);self.assertIn('Preview zoom',s);self.assertIn('response.dragged()',s);self.assertIn('events are not dispatched',s)
  durations=[.1,.2,.4];self.assertAlmostEqual(sum(durations[:2])+durations[2]/2,.5)
 def test_asset_safety(self):
  s=read('src/asset_pipeline.rs').split('pub fn move_asset',1)[1].split('pub fn import_folder',1)[0];self.assertIn('create_new(true)',s);self.assertNotIn('remove_file(&from)',s);self.assertNotIn('fs::rename(from,to)',s);self.assertIn('sound.path=new.into()',s);self.assertIn('Check dependencies',read('src/bin/forge_editor/master.rs'))
 def test_prefab_lighting_and_lock(self):
  self.assertIn('Add external child',read('src/bin/forge_editor/ultimate.rs'));self.assertIn('Remove link',read('src/bin/forge_editor/ultimate.rs'));self.assertIn('let normal=if lights.is_empty(){flat.clone()}',read('src/gpu_scene.rs'));v=tomllib.loads(read('Cargo.toml'))['package']['version'];self.assertEqual(v,'0.7.40');pat=re.search(r"\$pattern='([^']+)'",read('tools/prepare_lock.ps1')).group(1);raw='[[package]]\nname = "forge_engine"\nversion = "0.7.37"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(pat,r'\g<1>'+v+r'\g<2>',raw);self.assertEqual(n,1);self.assertIn('version = "0.7.40"',out);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
