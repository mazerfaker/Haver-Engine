"""Static integration checks and independent reference calculations; not Rust execution."""
import unittest, re, tomllib, math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def src(name):return (ROOT/name).read_text()
class Changes36(unittest.TestCase):
 def test_brand_scope(self):
  s=src('src/bin/forge_editor/max.rs');brand=s.split('pub(super) fn brand',1)[1].split('pub(super) fn menu',1)[0];self.assertNotIn('e.max',brand);self.assertIn('Sampled peak RSS',s);self.assertIn('env!("CARGO_PKG_VERSION")',src('src/bin/forge_editor.rs'))
 def test_live_track_gains(self):
  s=src('src/bin/forge_player/audio_dsp.rs');p=src('src/bin/forge_player.rs');self.assertIn('pub struct TrackGains',s);self.assertIn('self.at%self.channels as usize==0',s);self.assertIn('gains.set(i,gain)',p);self.assertIn('settings.summed_bus_effects(name)',p);self.assertAlmostEqual((1-math.exp(-1/(48000*.02))),.0010411243,places=8)
 def test_hierarchy_and_preview(self):
  s=src('src/pro_gameplay.rs');self.assertIn('pub fn summed_bus_effects',s);self.assertIn('fn parent_effects_combine',s);self.assertEqual(1-(1-.5)*(1-.5),.75);ui=src('src/bin/forge_editor/animation_studio.rs');self.assertIn('sample_clip_with_base',ui);self.assertIn('sprite_visual_channels(o,t)',ui);self.assertIn('o.animation.frame(t)',ui);self.assertIn('events are not dispatched',ui)
 def test_stable_graph_and_cache(self):
  s=src('src/prefab_studio.rs');self.assertIn('nested_id(id,c.id)',s);ui=src('src/bin/forge_editor/ultimate.rs');self.assertIn('row.id.to_string()',ui);self.assertIn('save_graph_layout(&layout,&positions)',ui);self.assertIn('recursive-layout.json',ui);self.assertIn('queued.len()>=32768',src('src/max_tools.rs'));self.assertIn('e.max.db_path!=e.path',src('src/bin/forge_editor/max.rs'))
 def test_lighting_and_lock(self):
  s=src('src/gpu_scene.rs');self.assertIn('frame_size[0].hypot(frame_size[1])',s);self.assertIn('images.remove(h.id())',s);ver=tomllib.loads(src('Cargo.toml'))['package']['version'];self.assertEqual(ver,'0.7.47');pat=re.search(r"\$pattern='([^']+)'",src('tools/prepare_lock.ps1')).group(1);old='[[package]]\nname = "forge_engine"\nversion = "0.7.35"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';new,n=re.subn(pat,r'\g<1>'+ver+r'\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.7.47"',new);self.assertIn('version = "0.16.1"',new)
if __name__=='__main__':unittest.main()
