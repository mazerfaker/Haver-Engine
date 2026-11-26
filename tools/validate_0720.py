"""Static fixtures, not Rust compilation or shader execution."""
import json,unittest
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
class Fixtures(unittest.TestCase):
 def test_quads_scene(self):
  p=json.loads((ROOT/'examples/gpu_quads/project.forge.json').read_text());assert p['settings']['gpu_particle_quads'] and p['settings']['gpu_particles'];assert sum(bool(o.get('pro',{}).get('emitter')) for o in p['objects'])==5
  shader=(ROOT/'src/gpu_particles_quads.wgsl').read_text();assert '@vertex' in shader and '@fragment' in shader;assert 'for(' not in shader and 'for (' not in shader;assert 'particles[min(slot,' in shader
 def test_regions(self):
  root=ROOT/'examples/atlas_regions';p=json.loads((root/'project.forge.json').read_text());o=next(o for o in p['objects'] if o.get('player'))
  with Image.open(root/o['sprite']) as img: w,h=img.size
  for a in [o['animation'],*o['clips'].values()]:
   r=a['regions'];assert len(r)==3;assert len({(v[2],v[3]) for v in r})==3
   for x,y,rw,rh in r:
    assert x>=0 and y>=0 and rw>0 and rh>0 and x+rw<=1 and y+rh<=1;assert all(abs(v-round(v))<1e-6 for v in (x*w,y*h,rw*w,rh*h))
   assert a['start']+a['frames']<=len(r)
 def test_lock_migration(self):
  import re
  s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.21"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"\n';out,n=re.subn(pat,r'\g<1>0.7.30\g<2>',old);assert n==1 and 'version = "0.7.30"' in out and 'version = "0.16.1"' in out
if __name__=='__main__':unittest.main()
