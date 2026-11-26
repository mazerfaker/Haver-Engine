"""Executable file transaction tests, plus static wiring checks (not Rust execution)."""
import json, re, tempfile, unittest, uuid, tomllib, subprocess, sys
from pathlib import Path
import relocate_asset as tx
import transfer_prefab as pt
ROOT=Path(__file__).resolve().parents[1]
class Transaction41(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory(); self.root=Path(self.tmp.name).resolve(); (self.root/'prefabs').mkdir(); (self.root/'assets').mkdir(); self.ids=[str(uuid.uuid4()) for _ in range(3)];a,b,c=self.ids
  self.a=self.root/'prefabs/a.prefab.json'; self.b=self.root/'prefabs/b.prefab.json';self.a.write_text(json.dumps([{'id':a,'parent':None,'position':[0,0]},{'id':b,'parent':a,'position':[3,4]}]));self.b.write_text(json.dumps([{'id':c,'parent':None,'position':[10,10]}]))
 def tearDown(self): self.tmp.cleanup()
 def test_cli_confirmed_transfer_and_gui_recovery(self):
  args=[sys.executable,str(ROOT/'tools/transfer_prefab.py'),str(self.root),'a.prefab.json','b.prefab.json',self.ids[1]];before=(self.a.read_bytes(),self.b.read_bytes());plan=subprocess.check_output(args,text=True);token=re.search(r'Plan SHA256: ([0-9a-f]{64})',plan).group(1);self.assertEqual(before,(self.a.read_bytes(),self.b.read_bytes()));out=subprocess.check_output(args+['--apply','--expect-plan',token],text=True);self.assertIn('Committed. Recovery journal:',out);j=next(self.root.glob('.forge-relocation-*'));self.assertEqual(len(json.loads(self.b.read_text())),2);subprocess.check_call([sys.executable,str(ROOT/'tools/relocate_asset.py'),str(self.root),'--rollback',str(j)]);self.assertEqual(before,(self.a.read_bytes(),self.b.read_bytes()))
 def test_transfer_rejects_stale_fingerprint(self):
  _,ops,_=pt.plan(self.root,'a.prefab.json','b.prefab.json',self.ids[1],None);token=pt.plan_token(ops);b=json.loads(self.b.read_text());b[0]['position']=[1,2];self.b.write_text(json.dumps(b));before=(self.a.read_bytes(),self.b.read_bytes());
  with self.assertRaisesRegex(RuntimeError,'plan changed'):pt.apply(self.root,'a.prefab.json','b.prefab.json',self.ids[1],expected_plan=token)
  self.assertEqual(before,(self.a.read_bytes(),self.b.read_bytes()));self.assertFalse(list(self.root.glob('.forge-relocation-*')))
 def test_rollback_rejects_duplicate_paths(self):
  old=self.root/'assets/a.png';old.write_bytes(b'test');j=tx.apply(self.root,'assets/a.png','assets/b.png');doc=json.loads((j/'journal.json').read_text());doc['entries'].append(doc['entries'][0]);(j/'journal.json').write_text(json.dumps(doc));
  with self.assertRaisesRegex(ValueError,'Invalid journal entries'):tx.rollback(self.root,j)
  self.assertFalse(old.exists());self.assertEqual((self.root/'assets/b.png').read_bytes(),b'test')
 def test_rollback_rejects_symlink_journal(self):
  old=self.root/'assets/a.png';old.write_bytes(b'test');j=tx.apply(self.root,'assets/a.png','assets/b.png');alias=self.root/'.forge-relocation-alias';
  try: alias.symlink_to(j,target_is_directory=True)
  except OSError: self.skipTest('symlinks unavailable')
  with self.assertRaisesRegex(ValueError,'Linked or aliased'):tx.rollback(self.root,alias)
  self.assertFalse(old.exists());tx.rollback(self.root,j);self.assertTrue(old.exists())
class Source41(unittest.TestCase):
 def test_editor_async_and_module(self):
  e=(ROOT/'src/bin/forge_editor.rs').read_text();m=(ROOT/'src/bin/forge_editor/master.rs').read_text();self.assertIn('#[path="forge_editor/animation_studio.rs"]',e);self.assertLess(e.index('master::transaction_tick(ctx,e)'),e.index('ultimate::recovery(ctx,e)'));self.assertIn('std::thread::Builder',m);self.assertIn('is_finished()',m);self.assertIn('Recover file transaction (journal)',m);self.assertIn('Confirm & apply',m);self.assertIn('super::master::prefab_transfer_tools(ui,e)',(ROOT/'src/bin/forge_editor/ultimate.rs').read_text())
 def test_timeline_meter_lighting(self):
  a=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text();self.assertIn('read_timeline_prefs',a);self.assertIn('read_curve_prefs',a);self.assertIn('Group Overshoot XY',a);self.assertIn('Group Ease In-Out XY',a);d=(ROOT/'src/bin/forge_player/audio_dsp.rs').read_text();self.assertIn('meter_window.flush',d);self.assertIn('self.rate/20',d);self.assertIn('rms_out',(ROOT/'src/bin/forge_player.rs').read_text());self.assertIn('else if samples==1u',(ROOT/'src/gpu_scene.wgsl').read_text());self.assertIn('normal_stamp(key)',(ROOT/'src/gpu_scene.rs').read_text())
 def test_lock_and_ci(self):
  ver=tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'];self.assertEqual(ver,'0.7.42');p=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.40"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';new,n=re.subn(p,r'\g<1>'+ver+r'\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.42"',new);self.assertIn('version = "0.16.1"',new);self.assertIn('HaverEngine-v0.7.42-Windows-portable.zip',(ROOT/'.github/workflows/windows.yml').read_text())
if __name__=='__main__':unittest.main()
