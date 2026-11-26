"""Executable file-transaction tests and independent static source checks, not Rust compilation."""
import tempfile, unittest, json, wave, math, struct, tomllib
from pathlib import Path
import relocate_asset as tx
ROOT=Path(__file__).resolve().parents[1]
class Transaction39(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory(); self.root=Path(self.tmp.name).resolve(); (self.root/'assets/new').mkdir(parents=True);(self.root/'prefabs').mkdir()
  self.old='assets/a.png';self.new='assets/new/b.png';(self.root/self.old).write_bytes(b'asset bytes')
  self.docs={'project.forge.json':{'objects':[{'sprite':self.old,'production':{'normal_map':self.old}}]},'prefabs/a.prefab.json':[{'sprite':self.old}],'assets/a.atlas.json':{'image':self.old},'assets/meta.atlas.json':{'image':self.old}}
  for p,v in self.docs.items():(self.root/p).write_text(json.dumps(v))
  (self.root/'.forge-assets.json').write_bytes(b'{}')
 def tearDown(self):self.tmp.cleanup()
 def snapshot(self):return {p.relative_to(self.root).as_posix():p.read_bytes() for p in self.root.rglob('*') if p.is_file() and not any(x.startswith('.forge-relocation-') for x in p.relative_to(self.root).parts)}
 def test_dry_run_commit_restore(self):
  before=self.snapshot();tx.plan(self.root,self.old,self.new);self.assertEqual(before,self.snapshot());j=tx.apply(self.root,self.old,self.new);self.assertFalse((self.root/self.old).exists());self.assertEqual((self.root/self.new).read_bytes(),b'asset bytes');self.assertFalse((self.root/'.forge-assets.json').exists());self.assertEqual(json.loads((self.root/'assets/new/b.atlas.json').read_text())['image'],self.new);self.assertIn(self.new,(self.root/'prefabs/a.prefab.json').read_text());tx.rollback(self.root,j);self.assertEqual(before,self.snapshot())
 def test_rollback_every_write(self):
  before=self.snapshot()
  for step in range(7):
   with self.assertRaises(RuntimeError):tx.apply(self.root,self.old,self.new,fail_after=step)
   self.assertEqual(before,self.snapshot())
 def test_no_overwrite_or_ambiguous(self):
  (self.root/self.new).write_bytes(b'existing');before=self.snapshot()
  with self.assertRaises(ValueError):tx.apply(self.root,self.old,self.new)
  self.assertEqual(before,self.snapshot());(self.root/self.new).unlink();(self.root/'script.lua').write_text('return "'+self.old+'"')
  with self.assertRaises(ValueError):tx.apply(self.root,self.old,self.new,True)
  self.assertTrue((self.root/self.old).exists())
 def test_dynamic_guard(self):
  (self.root/'script.lua').write_text('return prefix .. name')
  with self.assertRaises(ValueError):tx.plan(self.root,self.old,self.new)
  self.assertTrue(tx.plan(self.root,self.old,self.new,True)[-1])
 def test_rollback_conflict(self):
  j=tx.apply(self.root,self.old,self.new);(self.root/'project.forge.json').write_bytes(b'new user changes');before=self.snapshot()
  with self.assertRaises(RuntimeError):tx.rollback(self.root,j)
  self.assertEqual(before,self.snapshot())
class Source39(unittest.TestCase):
 def test_audio(self):
  d=(ROOT/'src/bin/forge_player/audio_dsp.rs').read_text();self.assertIn('Arc<[f32]>',d);self.assertIn('self.at+=1',d);self.assertIn('rate=rate.max(source.rate)',d);self.assertIn('channels=channels.max(source.channels)',d);self.assertIn('Resampled PCM exceeds budget',d);self.assertIn('fn mono_upmix_and_rate',d);self.assertAlmostEqual(struct.unpack('<f',struct.pack('<f',.1234567))[0],.1234567,places=7)
  p=ROOT/'examples/resampled_bus';project=json.loads((p/'project.forge.json').read_text());formats=[]
  for o in project['objects']:
   s=o.get('pro',{}).get('sound')
   if s:
    with wave.open(str(p/s['path'])) as w:formats.append((w.getframerate(),w.getnchannels(),w.getnframes()/w.getframerate()))
  self.assertEqual({f[:2] for f in formats},{(16000,1),(48000,2)});self.assertEqual(formats[0][2],formats[1][2])
 def test_preview_prefab_shader(self):
  s=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text();self.assertIn('Fade distant ghosts',s);self.assertIn('0..=5',s);self.assertIn('Preview height',s);self.assertIn('seen.contains(&frame)',s)
  s=(ROOT/'src/bin/forge_editor/ultimate.rs').read_text();self.assertIn('Replacement prefab path',s);self.assertIn('d.nested.swap(i,j)',s)
  s=(ROOT/'src/gpu_scene.wgsl').read_text();self.assertLess(s.index('if base.a<=0.0'),s.index('textureSampleLevel(normals'));self.assertIn('else{let perpendicular',s);self.assertIn('params.light.iter().all(|l|l.z<=0.)',(ROOT/'src/gpu_scene.rs').read_text())
if __name__=='__main__':unittest.main()
