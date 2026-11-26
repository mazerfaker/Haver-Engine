"""Static fixture checks; does not run the Rust engine or GPU."""
from pathlib import Path
import json, re, tomllib, unittest, uuid
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
class Fixtures(unittest.TestCase):
 def test_frame_pixels(self):
  r=ROOT/'examples/frame_geometry';p=json.loads((r/'project.forge.json').read_text());o=next(o for o in p['objects'] if o['player']);a=o['animation'];self.assertEqual(o['size'],[64,64]);self.assertEqual(a['frames'],2)
  with Image.open(r/o['sprite']) as im:
   def crop(region):
    x,y,w,h=region;return im.crop(tuple(round(v) for v in (x*im.width,y*im.height,(x+w)*im.width,(y+h)*im.height)))
   first=crop(a['regions'][0]);second=crop(a['regions'][1]);g=a['frame_geometry']['1'];self.assertEqual(first.getbbox(),(12,8,36,48));self.assertEqual(second.size,(24,40));self.assertEqual(first.crop(first.getbbox()).tobytes(),second.tobytes());self.assertEqual([round(v*64) for v in g],[12,8,24,40]);
 def test_nested_fixture(self):
  r=ROOT/'examples/frame_geometry/prefabs';p=json.loads((r/'parent.prefab.json').read_text());child=json.loads((r/p['nested'][0]['file']).read_text());self.assertEqual(p['nested'][0]['parent'],p['objects'][0]['id']);self.assertEqual(p['nested'][0]['offset'],[32,0]);self.assertNotEqual(uuid.UUID(child[0]['id']),uuid.UUID(p['objects'][0]['id']))
 def test_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.35');s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1)
  for v in range(14,27):
   old=f'[[package]]\nname = "forge_engine"\nversion = "0.7.{v}"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"\n';out,n=re.subn(pat,r'\g<1>0.7.35\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.35"',out);self.assertIn('version = "0.16.1"',out)
 def test_geometry_wiring(self):
  for file in ['src/bin/forge_editor.rs','src/bin/forge_editor/viewport.rs','src/bin/forge_player.rs','src/gpu_scene.rs']:
   self.assertIn('sprite_frame_geometry',(ROOT/file).read_text())
  self.assertIn('load_library_prefab',(ROOT/'src/bin/forge_editor/ultimate.rs').read_text())
if __name__=='__main__':unittest.main()
