"""Static retiming integration gate."""
import unittest,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Timeline45(unittest.TestCase):
 def test_wiring(self):
  core=(ROOT/'src/ultimate.rs').read_text(encoding='utf-8')
  editor=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text(encoding='utf-8')
  for term in ['pub fn scale_timeline_group','scale_preserves_shared_origin','scale_collision_is_atomic','scale_event_bounds_rollback']:self.assertIn(term,core)
  self.assertIn('forge_engine::scale_timeline_group',editor)
  self.assertIn('Apply time scale',editor)
 def test_package(self):
  version=tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version']
  self.assertEqual(version,'0.7.48')
  self.assertEqual([p['version'] for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package'] if p['name']=='forge_engine'],[version])
if __name__=='__main__':unittest.main()
