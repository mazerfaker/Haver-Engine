"""Static source/fixture checks; independent calculations, not Rust execution."""
import json, unittest, wave, re, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def src(p):return (ROOT/p).read_text()
class Changes37(unittest.TestCase):
 def test_cascade_fixture(self):
  d=ROOT/'examples/cascade_bus';p=json.loads((d/'project.forge.json').read_text());b=p['settings']['audio_buses'];self.assertTrue(b['Environment']['cascade_runtime']);self.assertEqual(b['Wet room']['parent'],'Environment');sounds=[o['pro']['sound'] for o in p['objects'] if 'sound' in o.get('pro',{})];self.assertEqual({s['custom_bus'] for s in sounds},{'Environment','Wet room'});formats=[]
  for s in sounds:
   with wave.open(str(d/s['path'])) as w:formats.append(w.getparams())
  self.assertEqual(formats[0],formats[1]);self.assertEqual((.5*.5+.5)*.5,.375)
  code=src('src/bin/forge_player/audio_dsp.rs');self.assertIn('self.scratch[parent]+=value',code);self.assertIn('fn child_then_parent_gain',code);self.assertIn('load_cascade',src('src/bin/forge_player.rs'))
 def test_preview(self):
  code=src('src/bin/forge_editor/animation_studio.rs');self.assertIn('sprite_frame_geometry_at(o,false,t,sample.0,sample.1)',code);self.assertIn('Mesh::with_texture',code);self.assertIn('o.animation.uv(t)',code);self.assertIn('events are not dispatched',code)
 def test_asset_prefab_gpu(self):
  self.assertIn('pub fn upstream_documents',src('src/max_tools.rs'));self.assertIn('write_dependency_cache',src('src/max_tools.rs'));self.assertIn('External child attachments',src('src/bin/forge_editor/ultimate.rs'));self.assertIn('RenderAssetUsages::RENDER_WORLD',src('src/gpu_scene.rs'));self.assertIn('p.light[2].z<=0.0',src('src/gpu_scene.wgsl'))
 def test_lock(self):
  v=tomllib.loads(src('Cargo.toml'))['package']['version'];self.assertEqual(v,'0.7.45');pat=re.search(r"\$pattern='([^']+)'",src('tools/prepare_lock.ps1')).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.36"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';new,n=re.subn(pat,r'\g<1>'+v+r'\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.45"',new);self.assertIn('version = "0.16.1"',new)
if __name__=='__main__':unittest.main()
