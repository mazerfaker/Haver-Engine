"""Static UI fixture and adaptive-layout reference checks, not a Rust/runtime test."""
from pathlib import Path
import json, unittest
ROOT=Path(__file__).resolve().parents[1]/'examples/ui_studio'
class UiFixture(unittest.TestCase):
 def setUp(self):
  self.scene=json.loads((ROOT/'project.forge.json').read_text())
 def test_fixture(self):
  s=self.scene['settings']; self.assertTrue(s['hud_canvas_groups']); self.assertFalse(s['gpu_particles']); self.assertFalse(s['gpu_lighting']); self.assertEqual(len(s['widgets']),6)
  self.assertEqual({w['action'] for w in s['widgets']},{'label','left','right','jump'})
  for o in self.scene['objects']:
   for rel in (o.get('sprite'),o.get('production',{}).get('normal_map')):
    if rel: self.assertTrue((ROOT/rel).is_file())
 def test_adaptive_reference(self):
  s=self.scene['settings']; design=s.get('production',{}).get('resolution',[640,360]);
  for width,height in [(1280,720),(1024,768),(720,1280),(320,180)]:
   for w in s['widgets']:
    scale=min(width/max(1,design[0]),height/max(1,design[1])); a,b=min(width,w['size'][0]*scale),min(height,w['size'][1]*scale)
    x,y=min(max(0,w['anchor'][0]*width),width-a),min(max(0,w['anchor'][1]*height),height-b)
    self.assertTrue(0<=x<=width-a and 0<=y<=height-b)
if __name__=='__main__':unittest.main()
