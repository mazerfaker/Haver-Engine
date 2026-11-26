//! Reusable opt-in combat components, independent of the demo.
use crate::{Object,Project};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Combat { pub team:u32,pub health:f32,pub max_health:f32,pub damage:f32,pub reach:[f32;2],pub knockback:f32,pub cooldown:f32,pub remaining:f32 }
impl Default for Combat { fn default()->Self {Self{team:0,health:3.,max_health:3.,damage:1.,reach:[105.,85.],knockback:24.,cooldown:0.35,remaining:0.}} }
/// Advances cooldowns once per simulation tick.
pub fn advance(p:&mut Project,dt:f32){for o in &mut p.objects {if let Some(c)=&mut o.pro.combat{c.remaining=(c.remaining-dt.max(0.)).max(0.);}}}
fn overlaps(a:[f32;2],half:[f32;2],o:&Object)->bool {(0..2).all(|i|(a[i]-o.position[i]).abs()<=half[i]+o.size[i]*0.5)}
/// One hit per invocation; deterministic nearest target, team filtering, directional hitbox.
pub fn strike(p:&mut Project,attacker:Uuid,direction:f32)->Option<Uuid>{
 strike_with_bonus(p,attacker,direction,0.)
}
pub fn strike_with_bonus(p:&mut Project,attacker:Uuid,direction:f32,bonus:f32)->Option<Uuid>{
 let a=p.objects.iter_mut().find(|o|o.id==attacker)?;let c=a.pro.combat.as_mut()?;
 if c.health<=0.||c.remaining>0.||!direction.is_finite()||direction.abs()<0.001{return None;}
 let dir=direction.signum();let pos=a.position;let reach=c.reach.map(|v|v.clamp(0.,10000.));let center=[pos[0]+dir*reach[0]*0.5,pos[1]];let half=[reach[0]*0.5,reach[1]*0.5];let team=c.team;let damage=(c.damage+if bonus.is_finite(){bonus.clamp(0.,1000.)}else{0.}).max(0.);let knock=c.knockback.max(0.);c.remaining=c.cooldown.max(0.01);
 let target=p.objects.iter_mut().filter(|o|o.id!=attacker&&o.color[3]>0&&o.pro.combat.as_ref().is_some_and(|c|c.health>0.&&c.team!=team)&&overlaps(center,half,o)).min_by(|a,b| (a.position[0]-pos[0]).abs().total_cmp(&(b.position[0]-pos[0]).abs()).then_with(||a.id.cmp(&b.id)))?;
 let c=target.pro.combat.as_mut()?;c.health=(c.health-damage).max(0.);if c.health==0.{target.color[3]=0;target.body=crate::Body::None;target.pro.light=None;}
 let alive=c.health>0.;let id=target.id;let mut proposed=target.clone();proposed.position[0]+=dir*knock;
 if alive&&!p.objects.iter().any(|o|o.id!=id&&o.body==crate::Body::Static&&overlaps(proposed.position,proposed.size.map(|v|v*0.5),o)){if let Some(o)=p.objects.iter_mut().find(|o|o.id==id){o.position=proposed.position;}}
 Some(id)
}
#[cfg(test)] mod tests {use super::*;#[test]fn teams_and_cooldown(){let mut p=Project::default();p.objects.clear();let mut a=Object::new("A",[0.,0.],[10.;2],[255;4]);a.pro.combat=Some(Combat::default());let id=a.id;let mut b=Object::new("B",[50.,0.],[10.;2],[255;4]);b.pro.combat=Some(Combat{team:1,..Default::default()});let bid=b.id;p.objects.extend([a,b]);assert_eq!(strike(&mut p,id,1.),Some(bid));assert_eq!(p.objects[1].pro.combat.as_ref().unwrap().health,2.);assert_eq!(strike(&mut p,id,1.),None);advance(&mut p,1.);assert_eq!(strike(&mut p,id,-1.),None);}}
