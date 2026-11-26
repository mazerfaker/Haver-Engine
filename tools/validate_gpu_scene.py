"""Static asset/config validation for the GPU fixture; no shader execution."""
from pathlib import Path
import json, uuid, unittest
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]/'examples/gpu_particles'
class Fixture(unittest.TestCase):
 def test_scene(self):
  p=json.loads((ROOT/'project.forge.json').read_text());o=p['objects']
  self.assertTrue(p['settings']['gpu_particles']);self.assertEqual(p['settings']['gpu_light_count'],3)
  self.assertGreater(p['settings']['shadow_softness'],0)
  self.assertEqual(len({uuid.UUID(x['id']) for x in o}),len(o));self.assertEqual(sum(bool(x.get('player')) for x in o),1)
  emit=[x['pro']['emitter'] for x in o if x.get('pro',{}).get('emitter')]
  self.assertEqual({e['effect'] for e in emit},{'Fire','Smoke','Sparks','Rain','Snow'})
  self.assertLessEqual(len(emit),8);self.assertGreaterEqual(sum(bool(x.get('pro',{}).get('light')) for x in o),3)
  for e in emit:
   self.assertGreater(e['rate'],0);self.assertGreater(e['lifetime'],0);self.assertGreater(e['size'],0)
   self.assertLessEqual(min(128,__import__('math').ceil(e['rate']*e['lifetime'])),128)
  for x in o:
   for rel in [x.get('sprite'),x.get('production',{}).get('normal_map'),x.get('production',{}).get('particle_texture')]:
    if rel:
     path=ROOT/rel;self.assertTrue(path.resolve().is_relative_to((ROOT/'assets').resolve()));self.assertTrue(path.is_file());
     with Image.open(path) as im: im.verify()
 def test_normal_dimensions(self):
  with Image.open(ROOT/'assets/panel.png') as a,Image.open(ROOT/'assets/panel_normal.png') as b:self.assertEqual(a.size,b.size)
if __name__=='__main__':unittest.main()
