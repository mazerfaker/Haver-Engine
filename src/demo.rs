//! Opt-in demo rules shared by editor and player. No changes to ordinary projects.
use crate::{Project, Simulation};
use std::collections::HashSet;
use uuid::Uuid;
#[derive(Default)]
pub struct DemoSession { collected: HashSet<Uuid>, pub won: bool, pub deaths: u32 }
impl DemoSession {
 pub fn score(&self)->usize {self.collected.len()}
 pub fn total(p:&Project)->usize {p.objects.iter().filter(|o|o.name.starts_with("Core:")).count()}
 pub fn update(&mut self,p:&mut Project) {
  if self.won {return;}
  let Some(player)=p.objects.iter().find(|o|o.player) else{return};
  let pos=player.position;
  if pos[1]< -340. {return;}
  let touching=|o:&crate::Object| (o.position[0]-pos[0]).abs()< (o.size[0]+player.size[0])*0.5 && (o.position[1]-pos[1]).abs()< (o.size[1]+player.size[1])*0.5;
  let total=Self::total(p);
  for o in &p.objects {if o.name.starts_with("Core:") && touching(o){self.collected.insert(o.id);}}
  self.won=total>0 && self.score()==total && p.objects.iter().any(|o|o.name=="Exit"&&touching(o));
  for o in &mut p.objects {if self.collected.contains(&o.id){o.color[3]=0; o.pro.light=None;}}
 }
}
/// Headless verification; no window, GPU or audio device required.
pub fn smoke(mut p:Project)->Result<(),String>{
 let initial=p.objects.iter().find(|o|o.player).ok_or("No player")?.position;
 let mut sim=Simulation::default();
 for _ in 0..120 {sim.step(&mut p,1./60.,[0.,0.]);}
 let settled=p.objects.iter().find(|o|o.player).ok_or("Player lost")?.position;
 if settled[1]>=initial[1] || settled[1]< -300. {return Err(format!("Gravity/ground failed: {initial:?} -> {settled:?}"));}
 for _ in 0..30 {sim.step(&mut p,1./60.,[1.,0.]);}
 let moved=p.objects.iter().find(|o|o.player).unwrap().position;
 if moved[0]<=settled[0]+10. {return Err("Horizontal movement failed".into());}
 if !p.objects.iter().all(|o|o.position.iter().all(|v|v.is_finite())) {return Err("Nonfinite position".into());}
 if !sim.errors.is_empty(){return Err(format!("Lua errors: {:?}",sim.errors));}
 let mut demo=DemoSession::default();
 let targets=p.objects.iter().filter(|o|o.name.starts_with("Core:")||o.name=="Exit").map(|o|(o.name.clone(),o.position)).collect::<Vec<_>>();
 for (_,pos) in targets {p.objects.iter_mut().find(|o|o.player).unwrap().position=pos;demo.update(&mut p);}
 if DemoSession::total(&p)>0 && !demo.won{return Err("Demo objective failed".into());}
 Ok(())
}
#[cfg(test)] mod tests {use super::*;#[test]fn objective(){let mut p=Project::default();let pos=p.objects.iter().find(|o|o.player).unwrap().position;p.objects.push(crate::Object::new("Core:1",pos,[10.;2],[255;4]));p.objects.push(crate::Object::new("Exit",pos,[10.;2],[255;4]));let mut d=DemoSession::default();d.update(&mut p);assert!(d.won);assert_eq!(d.score(),1);d.update(&mut p);assert_eq!(d.score(),1);}}
