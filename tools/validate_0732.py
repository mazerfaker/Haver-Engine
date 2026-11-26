"""Static wiring plus independent reference checks; not Rust compilation."""
import unittest,json,math,re,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_bezier(self):
  def cubic(t,a,b):return 3*(1-t)**2*t*a+3*(1-t)*t*t*b+t**3
  def evaluate(x,xx,yy):
   lo,hi=0.,1.
   for _ in range(24):
    t=(lo+hi)/2
    if cubic(t,*xx)<x:lo=t
    else:hi=t
   return cubic((lo+hi)/2,*yy)
  self.assertAlmostEqual(evaluate(.5,[0,0],[0,1]),.88988,places=3)
  self.assertAlmostEqual(evaluate(.25,[1/3,2/3],[1/3,2/3]),.25,places=5)
  core=(ROOT/'src/ultimate.rs').read_text();ui=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text()
  for x in ['bezier_ease(u,a.bezier_x','for _ in 0..24','pub bezier_x:Option<[f32;2]>','move_multi_channel','no channels changed']:self.assertIn(x,core)
  for x in ['xs[j]+delta.x','Move selected channels atomically','handles_changed']:self.assertIn(x,ui)
 def test_dependencies(self):
  d=ROOT/'examples/bezier_xy';p=json.loads((d/'project.forge.json').read_text());self.assertTrue(any(k.get('bezier_x') for o in p['objects'] for ch in o.get('motion',{}).get('property_channels',{}).values() for axis in ch for k in axis))
  graph=json.loads((d/'prefabs/root.prefab.json').read_text());leaf=json.loads((d/'prefabs'/graph['nested'][0]['file']).read_text());self.assertTrue((d/leaf[0]['sprite']).is_file())
  source=(ROOT/'src/max_tools.rs').read_text();self.assertIn('old.remove(&rel).filter(|d|Some(d.stamp)==stamp)',source);self.assertIn('seen.insert(node.clone())',source);self.assertIn('db.documents.len()<4096',source);self.assertIn('doc.nested',source)
 def test_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.43');pat=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1)
  raw='[[package]]\nname = "forge_engine"\nversion = "0.7.31"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(pat,r'\g<1>0.7.43\g<2>',raw);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
