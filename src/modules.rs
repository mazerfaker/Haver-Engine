//! Runtime capability checks: never silently discard required project features.
use crate::Project;
pub fn validate(p:&Project)->Result<(),String>{
 let mut missing=Vec::new();
 if !cfg!(feature="physics-rapier") && p.settings.rapier {missing.push("physics-rapier");}
 if !cfg!(feature="lua") && p.objects.iter().any(|o|!o.script.trim().is_empty()){missing.push("lua");}
 if !cfg!(feature="audio") && p.objects.iter().any(|o|o.pro.sound.is_some()){missing.push("audio");}
 if !cfg!(feature="lighting") && (p.settings.gpu_lighting || p.settings.gpu_particles || p.settings.ambient!=1. || p.objects.iter().any(|o|o.pro.light.is_some())){missing.push("lighting");}
 if missing.is_empty(){Ok(())}else{Err(format!("Project requires disabled features: {}",missing.join(", ")))}
}
#[cfg(test)] mod tests {use super::*;#[test] fn rapier_feature(){let mut p=Project::default();p.settings.rapier=true;assert_eq!(validate(&p).is_ok(),cfg!(feature="physics-rapier"));}}
