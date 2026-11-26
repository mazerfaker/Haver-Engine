use crate::{Project,Simulation};
use std::time::Instant;
pub fn benchmark(path:&std::path::Path,frames:usize)->Result<String,String>{
 let start=Instant::now();let mut p=Project::load(path)?;let load_ms=start.elapsed().as_secs_f64()*1000.;
 let mut sim=Simulation::default();let frames=frames.clamp(1,10000);let mut samples=Vec::with_capacity(frames);
 for _ in 0..60{sim.step(&mut p,1./60.,[0.,0.]);}
 for i in 0..frames{let t=Instant::now();sim.step(&mut p,1./60.,[if i%120<60{1.}else{-1.},0.]);samples.push(t.elapsed().as_secs_f64()*1000.);}
 samples.sort_by(f64::total_cmp);let mean=samples.iter().sum::<f64>()/frames as f64;
 Ok(serde_json::json!({"scene":path.to_string_lossy(),"objects":p.objects.len(),"frames":frames,"load_ms":load_ms,"simulation_mean_ms":mean,"simulation_p95_ms":samples[(frames-1)*95/100],"simulation_max_ms":samples[frames-1]}).to_string())
}
