"""Static reverse dependency integration gates."""
import unittest,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Reverse46(unittest.TestCase):
 def test_wiring(self):
  core=(ROOT/'src/max_tools.rs').read_text(encoding='utf-8')
  ui=(ROOT/'src/bin/forge_editor/max.rs').read_text(encoding='utf-8')
  for term in ['pub fn reverse_dependencies','reverse_graph_distances_and_cycles','reverse_missing_and_duplicate_edges','262144']:self.assertIn(term,core)
  self.assertIn('reverse_impact_panel(ui,db)',ui)
  self.assertIn('db.reverse_dependencies(&query)',ui)
  self.assertGreaterEqual(ui.count('>(egui::Id::new("reverse_result46"))'),2)
 def test_package(self):
  version=tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version']
  self.assertEqual(version,'0.7.48')
  self.assertEqual([p['version'] for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package'] if p['name']=='forge_engine'],[version])
if __name__=='__main__':unittest.main()
