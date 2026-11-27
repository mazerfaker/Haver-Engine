"""Static fixtures and source checks, not Rust/GPU execution."""
import unittest,json,re,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Checks(unittest.TestCase):
 def test_fixture(self):
  p=json.loads((ROOT/'examples/inventory_pro/project.forge.json').read_text());s=p['settings'];self.assertEqual(len(s['inventory']['slots']),12);self.assertEqual(s['item_catalog']['Training sword']['equip_slot'],'weapon');self.assertEqual(s['item_catalog']['Medkit']['heal'],2);self.assertEqual(next(o for o in p['objects'] if o['player'])['pro']['combat']['health'],1);self.assertEqual(s['item_catalog']['Energy cell']['variable'],'energy');self.assertTrue(all(i['item'] in s['item_catalog'] for i in s['inventory']['slots'] if i))
 def test_version_lock(self):
  p=tomllib.loads((ROOT/'Cargo.toml').read_text());self.assertGreaterEqual(tuple(map(int,p['package']['version'].split('.'))),(0,7,23));s=(ROOT/'tools/prepare_lock.ps1').read_text();pat=re.search(r"\$pattern='([^']+)'",s).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.22"\n\n[[package]]\nname = "bevy"\nversion = "0.16.1"\n';new,n=re.subn(pat,r'\g<1>0.7.47\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.47"',new);self.assertIn('version = "0.16.1"',new)
 def test_checkpoint_wiring(self):
  s=(ROOT/'src/bin/forge_player.rs').read_text();self.assertIn('KeyCode::F5',s);self.assertIn('KeyCode::F9',s);self.assertGreaterEqual(s.count('slot_path(&game.entry,game.checkpoint_slot)'),2);save=(ROOT/'src/save_game.rs').read_text();self.assertIn('1=>Ok(entry.with_extension("checkpoint.json"))',save);self.assertIn('checkpoint.slot{slot}.json',save);self.assertIn('game.sim.elapsed=c.elapsed',s);self.assertIn('strike_with_bonus',s)
if __name__=='__main__':unittest.main()
