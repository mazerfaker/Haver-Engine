"""Executable transaction regressions + independent DSP numeric reference and static wiring checks."""
import json,math,re,tempfile,unittest,uuid,tomllib
from pathlib import Path
import relocate_asset as tx
import transfer_prefab as pt
ROOT=Path(__file__).resolve().parents[1]
class Transfer40(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.root=Path(self.tmp.name).resolve();(self.root/'prefabs').mkdir();(self.root/'assets/new').mkdir(parents=True)
  self.ids=[str(uuid.uuid4()) for _ in range(4)];a,b,c,d=self.ids
  self.a=self.root/'prefabs/a.prefab.json';self.b=self.root/'prefabs/b.prefab.json'
  self.a.write_text(json.dumps([{'id':a,'parent':None,'position':[0,0]},{'id':b,'parent':a,'position':[3,4]},{'id':c,'parent':b,'position':[5,6]}]));self.b.write_text(json.dumps({'objects':[{'id':d,'parent':None,'position':[10,20]}],'nested':[],'custom':'keep'}))
 def tearDown(self):self.tmp.cleanup()
 def snapshot(self):return self.a.read_bytes(),self.b.read_bytes()
 def test_transfer_commit_rollback(self):
  before=self.snapshot();j,n=pt.apply(self.root,'a.prefab.json','b.prefab.json',self.ids[1],self.ids[3]);self.assertEqual(n,2);a,b=json.loads(self.a.read_text()),json.loads(self.b.read_text());self.assertEqual(len(a),1);self.assertEqual(b['custom'],'keep');self.assertEqual(b['objects'][1]['parent'],self.ids[3]);self.assertEqual(b['objects'][2]['parent'],self.ids[1]);self.assertEqual(b['objects'][1]['position'],[3,4]);tx.rollback(self.root,j);self.assertEqual(before,self.snapshot())
 def test_transfer_injected_failure(self):
  before=self.snapshot()
  for i in range(2):
   with self.assertRaises(RuntimeError):pt.apply(self.root,'a.prefab.json','b.prefab.json',self.ids[1],fail_after=i)
   self.assertEqual(before,self.snapshot())
 def test_transfer_refuses_consumers(self):
  (self.root/'project.forge.json').write_text(json.dumps({'objects':[{'prefab_link':{'file':'a.prefab.json'}}]}));before=self.snapshot()
  with self.assertRaises(ValueError):pt.apply(self.root,'a.prefab.json','b.prefab.json',self.ids[1]);
  self.assertEqual(before,self.snapshot())
 def test_relocation_atlas_state_and_approval(self):
  old='assets/a.png';new='assets/new/b.png';(self.root/old).write_bytes(b'a');meta=self.root/f'assets/atlas-state-{self.ids[0]}.json';meta.write_text(json.dumps({'schema':2,'object':self.ids[0],'image':old}));_,asset,changes,side,warnings=tx.plan(self.root,old,new);token=tx.plan_digest(old,new,asset,changes,side,warnings);(self.root/old).write_bytes(b'changed')
  with self.assertRaises(RuntimeError):tx.apply(self.root,old,new,expected_plan=token)
  self.assertTrue((self.root/old).is_file());self.assertFalse((self.root/new).exists());(self.root/old).write_bytes(b'a');j=tx.apply(self.root,old,new,expected_plan=token);self.assertEqual(json.loads(meta.read_text())['image'],new);tx.rollback(self.root,j);self.assertEqual(json.loads(meta.read_text())['image'],old)
class Source40(unittest.TestCase):
 def test_preview_and_relocation_wiring(self):
  a=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text();canvas=a.split('fn atlas_canvas',1)[1].split('fn ',1)[0];self.assertNotIn('id.with("onion39")',canvas);self.assertIn('struct PreviewPrefs',a);self.assertIn('Save preview preferences',a);self.assertIn('Zoom selection',a);self.assertIn('Center playhead',a);self.assertIn('prefix("X ")',a);master=(ROOT/'src/bin/forge_editor/master.rs').read_text();self.assertIn('Transactional Move / Rename',master);self.assertIn('--expect-plan',master);self.assertIn('Project::load(&path)',master);self.assertIn('e.external_reload=true',master)
 def test_dsp_shader_wiring(self):
  a=(ROOT/'src/bin/forge_player/audio_dsp.rs').read_text();self.assertIn('sinc_kernel',a);self.assertIn('self.effects[i].process_unclipped',a);self.assertIn('self.limiter_gain',a);self.assertIn('PeakMeter',a);self.assertIn('Shared bus peaks / limiter',(ROOT/'src/bin/forge_player.rs').read_text());w=(ROOT/'src/gpu_scene.wgsl').read_text();self.assertIn('p.light_color[0].w>0.5',w);self.assertLess(w.index('if diffuse<=0.0'),w.index('var visible'));self.assertIn('normal_map_rgba8_payload_bytes',(ROOT/'src/bin/forge_editor/max.rs').read_text())
 def test_resample_reference(self):
  # Independent numerical reference, not an execution of Rust. Reject 16 kHz when downsampling 48 -> 16 kHz.
  cutoff=16000/48000*.94;weights=[]
  for j in range(32):
   x=j-15;a=math.pi*x*cutoff;win=0 if abs(x)>=16 else .42+.5*math.cos(math.pi*x/16)+.08*math.cos(math.tau*x/16);weights.append(cutoff*(1 if abs(a)<1e-10 else math.sin(a)/a)*win)
  weights=[x/sum(weights) for x in weights];self.assertAlmostEqual(sum(weights),1);response=abs(sum(w*complex(math.cos(math.tau*16000/48000*(j-15)),math.sin(math.tau*16000/48000*(j-15))) for j,w in enumerate(weights)));self.assertLess(response,.05)
 def test_version_lock(self):
  version=tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'];self.assertEqual(version,'0.7.44');s=(ROOT/'tools/prepare_lock.ps1').read_text();pattern=re.search(r"\$pattern='([^']+)'",s).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.39"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"';new,n=re.subn(pattern,r'\g<1>'+version+r'\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.44"',new);self.assertIn('version = "0.16.1"',new)
if __name__=='__main__':unittest.main()
