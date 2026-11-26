"""Save slot wiring and package consistency; Rust tests execute in cargo test."""
import unittest,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def read(p):return (ROOT/p).read_text(encoding='utf-8')
class Save44(unittest.TestCase):
 def test_slot_wiring(self):
  s=read('src/save_game.rs');p=read('src/bin/forge_player.rs')
  for term in ['SLOT_COUNT:usize=3','checkpoint.slot{slot}.json','independent_slots_and_corruption','slot_paths_preserve_legacy_and_reject_invalid']:self.assertIn(term,s)
  self.assertEqual(p.count('slot_path(&game.entry,game.checkpoint_slot)'),2)
  self.assertIn('checkpoint_slot:1',p)
  self.assertIn('ui.selectable_value(&mut game.checkpoint_slot',p)
 def test_version_and_base(self):
  version=tomllib.loads(read('Cargo.toml'))['package']['version']
  self.assertEqual(version,'0.7.45')
  self.assertEqual([p['version'] for p in tomllib.loads(read('Cargo.lock'))['package'] if p['name']=='forge_engine'],[version])
  self.assertEqual(read('docs/PATCH_BASE.txt').strip(),'98c8eba9153d02219d716369dd7470c86c7f3154')
if __name__=='__main__':unittest.main()
