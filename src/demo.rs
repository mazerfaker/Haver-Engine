//! Opt-in demo rules shared by editor and player. No changes to ordinary projects.
use crate::{Project, Simulation};
use std::collections::HashSet;
use uuid::Uuid;
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct DemoSession { collected: HashSet<Uuid>, pub won: bool, pub deaths: u32, pub health: u8, pub cooldown: f32, pub attack_cooldown: f32, pub facing: f32 }
impl Default for DemoSession { fn default()->Self { Self{collected:HashSet::new(),won:false,deaths:0,health:3,cooldown:0.,attack_cooldown:0.,facing:1.} } }
impl DemoSession {
 pub fn score(&self)->usize {self.collected.len()}
 pub fn total(p:&Project)->usize {p.objects.iter().filter(|o|o.name.starts_with("Core:")).count()}
 pub fn face(&mut self,x:f32){if x.abs()>0.01 {self.facing=x.signum();}}
 pub fn attack(&mut self,p:&mut Project)->bool {
  if self.won || self.health==0 || self.attack_cooldown>0. {return false;}
  self.attack_cooldown=0.35;
  let Some(pos)=p.objects.iter().find(|o|o.player).map(|o|o.position) else{return false;};
  let target=p.objects.iter_mut().filter(|o| (o.name.starts_with("Enemy:") || o.name.starts_with("Boss:")) && o.color[3]>0 && (o.position[0]-pos[0])*self.facing >= -12. && (o.position[0]-pos[0]).abs()<105. && (o.position[1]-pos[1]).abs()<85.).min_by(|a,b| (a.position[0]-pos[0]).abs().total_cmp(&(b.position[0]-pos[0]).abs()));
  if let Some(o)=target {let hp=o.pro.variables.entry("combat_hp".into()).or_insert(if o.name.starts_with("Boss:"){6.}else if o.name.starts_with("Enemy:Heavy"){4.}else{2.});*hp-=1.;if *hp<=0.{o.color[3]=0;o.body=crate::Body::None;o.pro.light=None;}return true;}false
 }
 pub fn update(&mut self,p:&mut Project) {
  self.step(p,1./60.);
 }
 pub fn step(&mut self,p:&mut Project,dt:f32) {
  if self.won || self.health==0 {return;}
  self.attack_cooldown=(self.attack_cooldown-dt.max(0.)).max(0.);
  if self.attack_cooldown>0.12 {if let Some(o)=p.objects.iter_mut().find(|o|o.player){if let Some(clip)=o.clips.get("Attack"){o.animation=clip.clone();o.clip_name="Attack".into();o.pro.variables.insert("forge_attack_time".into(),0.35-self.attack_cooldown);}}}
  let player_pos=p.objects.iter().find(|o|o.player).map(|o|o.position);
  if let Some(pos)=player_pos {for o in &mut p.objects {if o.name.starts_with("Boss:")&&o.color[3]>0 {let hp=*o.pro.variables.get("combat_hp").unwrap_or(&6.);let phase=if hp<=3. {2.}else{1.};o.pro.variables.insert("combat_phase".into(),phase);let dx=pos[0]-o.position[0];if dx.abs()>24. {o.position[0]+=(dx.signum()*if phase>1. {95.}else{38.}*dt).clamp(-dx.abs(),dx.abs());} }}}
  self.cooldown=(self.cooldown-dt.max(0.)).max(0.);
  for o in &mut p.objects {if o.name.starts_with("Enemy:") && o.color[3]>0 {let dir=o.pro.variables.entry("patrol_direction".into()).or_insert(1.);o.position[0]+=*dir*if o.name.starts_with("Enemy:Heavy"){42.}else{70.}*dt;if o.position[0]>410.{o.position[0]=410.;*dir=-1.;}else if o.position[0]< -320.{o.position[0]=-320.;*dir=1.;}}}
  let Some(player)=p.objects.iter().find(|o|o.player) else{return};
  let pos=player.position;
  if pos[1]< -340. {self.health=0;self.deaths+=1;return;}
  let touching=|o:&crate::Object| (o.position[0]-pos[0]).abs()< (o.size[0]+player.size[0])*0.5 && (o.position[1]-pos[1]).abs()< (o.size[1]+player.size[1])*0.5;
  if self.cooldown<=0. && p.objects.iter().any(|o|(o.name.starts_with("Enemy:") || o.name.starts_with("Boss:")) && o.color[3]>0 && touching(o)){self.health=self.health.saturating_sub(1);self.cooldown=1.2;if self.health==0{self.deaths+=1;return;}}
  let total=Self::total(p);
  for o in &p.objects {if o.name.starts_with("Core:") && touching(o){self.collected.insert(o.id);}}
  self.won=total>0 && self.score()==total && !p.objects.iter().any(|o|o.name.starts_with("Boss:") && o.color[3]>0) && p.objects.iter().any(|o|o.name=="Exit"&&touching(o));
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
 for o in &mut p.objects {if o.name.starts_with("Boss:"){o.color[3]=0;}}
 let targets=p.objects.iter().filter(|o|o.name.starts_with("Core:")||o.name=="Exit").map(|o|(o.name.clone(),o.position)).collect::<Vec<_>>();
 for (_,pos) in targets.iter().filter(|(n,_)|n.starts_with("Core:")).chain(targets.iter().filter(|(n,_)|n=="Exit")) {p.objects.iter_mut().find(|o|o.player).unwrap().position=*pos;demo.update(&mut p);}
 if DemoSession::total(&p)>0 && !demo.won{return Err("Demo objective failed".into());}
 Ok(())
}
#[cfg(test)] mod tests {use super::*;#[test]fn objective(){let mut p=Project::default();let pos=p.objects.iter().find(|o|o.player).unwrap().position;p.objects.push(crate::Object::new("Core:1",pos,[10.;2],[255;4]));p.objects.push(crate::Object::new("Exit",pos,[10.;2],[255;4]));let mut d=DemoSession::default();d.update(&mut p);assert!(d.won);assert_eq!(d.score(),1);d.update(&mut p);assert_eq!(d.score(),1);}}

#[cfg(test)] mod pro_tests {use super::*;#[test]fn damage_cooldown(){let mut p=Project::default();let pos=p.objects.iter().find(|o|o.player).unwrap().position;p.objects.push(crate::Object::new("Enemy:1",pos,[20.;2],[255;4]));let mut d=DemoSession::default();d.step(&mut p,0.);assert_eq!(d.health,2);d.step(&mut p,0.);assert_eq!(d.health,2);d.cooldown=0.;d.step(&mut p,0.);assert_eq!(d.health,1);}#[test]fn exit_requires_cores(){let mut p=Project::default();let pos=p.objects.iter().find(|o|o.player).unwrap().position;p.objects.push(crate::Object::new("Core:1",[900.,900.],[10.;2],[255;4]));p.objects.push(crate::Object::new("Exit",pos,[10.;2],[255;4]));let mut d=DemoSession::default();d.update(&mut p);assert!(!d.won);}}

#[cfg(test)] mod combat_tests {use super::*; #[test] fn boss_requires_hits(){let mut p=Project::default();let pos=p.objects.iter().find(|o|o.player).unwrap().position;p.objects.push(crate::Object::new("Boss:Test",pos,[30.;2],[255;4]));let mut d=DemoSession::default();for _ in 0..6 {d.attack_cooldown=0.;assert!(d.attack(&mut p));}assert_eq!(p.objects.last().unwrap().color[3],0);}}

#[cfg(test)] mod refinement_tests {use super::*; #[test] fn facing_blocks_behind(){let mut p=Project::default();let pos=p.objects.iter().find(|o|o.player).unwrap().position;p.objects.push(crate::Object::new("Enemy:Heavy",[pos[0]-60.,pos[1]],[20.;2],[255;4]));let mut d=DemoSession::default();assert!(!d.attack(&mut p));d.attack_cooldown=0.;d.face(-1.);assert!(d.attack(&mut p));assert_eq!(p.objects.last().unwrap().pro.variables["combat_hp"],3.);}}
