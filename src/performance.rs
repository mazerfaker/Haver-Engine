use crate::{Project,Simulation};
use std::time::Instant;
pub fn benchmark(path:&std::path::Path,frames:usize)->Result<String,String>{
 let start=Instant::now();let mut p=Project::load(path)?;crate::modules::validate(&p)?;let load_ms=start.elapsed().as_secs_f64()*1000.;
 let mut sim=Simulation::default();let frames=frames.clamp(1,10000);let mut samples=Vec::with_capacity(frames);
 for _ in 0..60{sim.step(&mut p,1./60.,[0.,0.]);}
 for i in 0..frames{let t=Instant::now();sim.step(&mut p,1./60.,[if i%120<60{1.}else{-1.},0.]);samples.push(t.elapsed().as_secs_f64()*1000.);}
 samples.sort_by(f64::total_cmp);let mean=samples.iter().sum::<f64>()/frames as f64;
 Ok(serde_json::json!({"scene":path.to_string_lossy(),"objects":p.objects.len(),"frames":frames,"load_ms":load_ms,"simulation_mean_ms":mean,"simulation_p95_ms":samples[(frames-1)*95/100],"simulation_max_ms":samples[frames-1]}).to_string())
}

#[derive(Clone,Copy,Debug,PartialEq,serde::Serialize)]
pub struct FrameSummary{pub samples:usize,pub mean_ms:f64,pub median_ms:f64,pub p95_ms:f64,pub p99_ms:f64,pub max_ms:f64,pub estimated_fps:f64}
/// Frame intervals, not GPU timestamps. Ignore nonpositive and nonfinite samples.
pub fn frame_summary(samples:impl IntoIterator<Item=f64>)->Option<FrameSummary>{
 let mut s:Vec<_>=samples.into_iter().filter(|v|v.is_finite()&&*v>0.).collect();if s.is_empty(){return None;}
 s.sort_by(f64::total_cmp);let n=s.len();let mean=s.iter().sum::<f64>()/n as f64;if !mean.is_finite(){return None;}
 let pick=|p:usize|s[(n-1)*p/100];
 Some(FrameSummary{samples:n,mean_ms:mean,median_ms:pick(50),p95_ms:pick(95),p99_ms:pick(99),max_ms:s[n-1],estimated_fps:1000./mean})
}
#[cfg(test)]mod summary48{use super::*;#[test]fn samples_and_invalid_values(){let s=frame_summary([10.,20.,30.,40.,f64::NAN,0.,-5.]).unwrap();assert_eq!(s.samples,4);assert_eq!(s.mean_ms,25.);assert_eq!(s.estimated_fps,40.);assert_eq!(s.max_ms,40.);assert_eq!(s.p95_ms,30.);assert!(frame_summary([0.,f64::INFINITY]).is_none());}}
