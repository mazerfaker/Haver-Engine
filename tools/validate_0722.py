"""Static checks only; Rust and GPU execution require Windows build."""
import json,re,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Fixtures(unittest.TestCase):
 def test_inventory(self):
  p=json.loads((ROOT/'examples/inventory/project.forge.json').read_text());o=p['objects'];inv=p['settings']['inventory'];self.assertEqual(len(inv['slots']),12);self.assertEqual(len({v['id'] for v in o}),len(o));self.assertEqual(sum('pickup' in v.get('pro',{}) for v in o),3);self.assertEqual(sum(v.get('player',False) for v in o),1);self.assertEqual(inv['slots'][0]['count'],2);self.assertEqual(inv['collected'],[]);self.assertTrue(all(1<=v['pro']['pickup']['count']<=99 for v in o if 'pickup' in v.get('pro',{})))
 def test_events(self):
  p=json.loads((ROOT/'examples/inventory/project.forge.json').read_text());o=next(v for v in p['objects'] if v['player']);e=o['motion']['events'][0];self.assertEqual(e['clip'],o['clip_name']);self.assertTrue(0<=e['time']<o['animation']['frames']/o['animation']['fps'])
 def test_indexed_quad(self):
  s=(ROOT/'src/gpu_particles.rs').read_text();self.assertIn('indices.extend([b,b+1,b+2,b,b+2,b+3])',s);self.assertIn('bevy::render::mesh::Indices::U16(indices)',s);self.assertIn('Vec::with_capacity(n*4)',s)
if __name__=='__main__':unittest.main()
