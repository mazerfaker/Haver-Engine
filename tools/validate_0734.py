"""Static wiring and reference-model checks, not execution of Rust."""
import unittest,re,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_clipboard(self):
  tracks=[[1.,2.],[1.5]];events=[1.25];first=min([*tracks[0],*tracks[1],*events]);self.assertEqual([[4+t-first for t in lane] for lane in tracks],[[4.,5.],[4.5]])
  core=(ROOT/'src/ultimate.rs').read_text();ui=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text()
  for x in ['pub struct TimelineClipboard','let mut draft=m.clone()','*m=draft','event collision']:
   self.assertIn(x.lower(),core.lower())
  for x in ['Copy selected lanes + events','Paste group at playhead','Scroll seconds','event34']:self.assertIn(x,ui)
 def test_bus(self):
  self.assertAlmostEqual(.2*.5+.4*.5,.3);s=(ROOT/'src/audio_effects.rs').read_text();self.assertIn('self.effects.process(out)',s);self.assertIn('pub fn mix_bus_wavs',s);self.assertIn('Export summed WAV (offline)',(ROOT/'src/bin/forge_editor/gameplay_ui.rs').read_text())
 def test_graphs_and_buffers(self):
  self.assertIn('transitive_cache34',(ROOT/'src/bin/forge_editor/max.rs').read_text());self.assertIn('inline_nested_edit',(ROOT/'src/bin/forge_editor/ultimate.rs').read_text());s=(ROOT/'src/gpu_scene.rs').read_text();self.assertIn('std::mem::take(&mut self.state.solid_scratch)',s);self.assertIn('materials.remove(v.material.id())',s)
 def test_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.40');pat=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.33"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(pat,r'\g<1>0.7.40\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
