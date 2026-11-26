"""Static wiring and reference checks, not Rust execution."""
import unittest, re, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Changes(unittest.TestCase):
 def test_timeline(self):
  ui=(ROOT/'src/bin/forge_editor/animation_studio.rs').read_text(); core=(ROOT/'src/ultimate.rs').read_text()
  for term in ['Rectangle selection','marquee_id','rect.contains(pos(k.time,k.value))','move_scalar_group(keys,&group,delta,span)','Curve presets']:
   self.assertIn(term,ui)
  for term in ['preserves_spacing_and_rejects_collision','!group.contains(&j)','k.time+=delta']:
   self.assertIn(term,core)
 def test_asset_cache_and_dependencies(self):
  src=(ROOT/'src/asset_pipeline.rs').read_text();db=(ROOT/'src/max_tools.rs').read_text(); ui=(ROOT/'src/bin/forge_editor/max.rs').read_text()
  for term in ['.forge-import-cache-v1.json','cache_roundtrip_and_size_invalidation','t.subsec_nanos()','persistent.lookup','bytes==raw','4*1024*1024']:
   self.assertIn(term,src)
  for term in ['scan_warnings','Could not parse scene','Could not parse prefab','"wav"|"ogg"']:
   self.assertIn(term,db)
  for term in ['Potentially unused','Reverse dependency graph','Diagnostic only']:
   self.assertIn(term,ui)
 def test_gpu_measurement_and_graph(self):
  for f,terms in {'src/bin/forge_editor.rs':['FORGE_GPU_PROFILE','RenderDiagnosticsPlugin','diagnostics.as_deref()'],'src/gpu_particles.rs':['diagnostic_recorder()','forge_particle_compute','span.end(ctx.command_encoder())'],'src/bin/forge_editor/max.rs':['/elapsed_gpu','"gpu_pass_ms":gpu_pass_ms','"gpu_frame_ms":null','"total_vram_bytes":null','sampled_peak_rss_bytes','process_cpu_percent'],'src/bin/forge_editor/ultimate.rs':['recursive_node31','ancestors.truncate(depth)','Open selected dependency (discard current draft)']}.items():
   for term in terms:self.assertIn(term,(ROOT/f).read_text())
 def test_lock(self):
  self.assertEqual(tomllib.loads((ROOT/'Cargo.toml').read_text())['package']['version'],'0.7.40');p=re.search(r"\$pattern='([^']+)'",(ROOT/'tools/prepare_lock.ps1').read_text()).group(1)
  for version in range(14,31):
   old=f'[[package]]\nname = "forge_engine"\nversion = "0.7.{version}"\n[[package]]\nname = "bevy"\nversion = "0.16.1"';out,n=re.subn(p,r'\g<1>0.7.40\g<2>',old);self.assertEqual(n,1);self.assertIn('version = "0.16.1"',out)
if __name__=='__main__':unittest.main()
