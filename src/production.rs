//! Portable production tools. Coordinates are in world pixels.
use crate::{Project,Body,Object};
use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap,HashMap,HashSet,VecDeque};
use uuid::Uuid;
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)]
pub struct Controls {pub left:String,pub right:String,pub up:String,pub down:String,pub deadzone:f32}
impl Default for Controls {fn default()->Self{Self{left:"A".into(),right:"D".into(),up:"W".into(),down:"S".into(),deadzone:0.2}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)]
pub struct Navigation {pub target:[f32;2],pub speed:f32,pub grid:u32,pub enabled:bool}
impl Default for Navigation{fn default()->Self{Self{target:[0.,0.],speed:80.,grid:32,enabled:true}}}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)] #[serde(default)]
pub struct ProductionObject {pub navigation:Option<Navigation>,pub particle_texture:Option<String>,pub normal_map:Option<String>}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)]
pub struct ProductionSettings {pub controls:Controls,pub resolution:[u32;2],pub package_id:String,pub joints:Vec<DistanceJoint>}
impl Default for ProductionSettings{fn default()->Self{Self{controls:Controls::default(),resolution:[1280,720],package_id:"org.forge.game".into(),joints:vec![]}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] pub struct DistanceJoint{pub a:Uuid,pub b:Uuid,pub length:f32}
/// Sparse chunk index; negative coordinates use Euclidean division.
pub fn tile_chunks(p:&Project)->BTreeMap<(i32,u32,i32,i32),Vec<Uuid>>{let mut out=BTreeMap::<_,Vec<_>>::new();for o in &p.objects{if let Some(t)=&o.tile{out.entry((t.layer,t.grid,t.cell[0].div_euclid(16),t.cell[1].div_euclid(16))).or_default().push(o.id);}}out}
/// Bounded shortest-path search on a four-connected grid. No diagonal corner cutting.
pub fn pathfind(start:[i32;2],goal:[i32;2],blocked:&HashSet<[i32;2]>,budget:usize)->Option<Vec<[i32;2]>>{if blocked.contains(&goal){return None;}let mut q=VecDeque::from([start]);let mut prev=HashMap::from([(start,start)]);let mut visited=0;while let Some(c)=q.pop_front(){if c==goal{let mut path=vec![c];let mut x=c;while x!=start{x=prev[&x];path.push(x);}path.reverse();return Some(path);}visited+=1;if visited>=budget.min(65536){return None;}for d in [[1,0],[-1,0],[0,1],[0,-1]]{let Some(x)=c[0].checked_add(d[0])else{continue};let Some(y)=c[1].checked_add(d[1])else{continue};let n=[x,y];if !blocked.contains(&n)&&!prev.contains_key(&n){prev.insert(n,c);q.push_back(n);}}}None}
pub fn navigate(o:&mut Object,world:&Project,dt:f32){let Some(n)=o.production.navigation.as_ref()else{return};if !n.enabled||o.body!=Body::None||o.player{return;}let g=n.grid.clamp(8,256) as f32;let cell=|p:[f32;2]|p.map(|v|(v/g).round() as i32);let mut blocked=HashSet::new();for b in world.objects.iter().filter(|b|b.body==Body::Static){let lo=cell([b.position[0]-b.size[0]/2.-o.size[0]/2.,b.position[1]-b.size[1]/2.-o.size[1]/2.]);let hi=cell([b.position[0]+b.size[0]/2.+o.size[0]/2.,b.position[1]+b.size[1]/2.+o.size[1]/2.]);if i64::from(hi[0])-i64::from(lo[0])>128||i64::from(hi[1])-i64::from(lo[1])>128{return;}for x in lo[0]..=hi[0]{for y in lo[1]..=hi[1]{blocked.insert([x,y]);}}}let Some(path)=pathfind(cell(o.position),cell(n.target),&blocked,2048)else{return};let target=path.get(1).map(|c|c.map(|v|v as f32*g)).unwrap_or(n.target);let d=[target[0]-o.position[0],target[1]-o.position[1]];let len=d[0].hypot(d[1]);if len>0.001{let step=(n.speed.max(0.)*dt).min(len)/len;for i in 0..2{o.position[i]+=d[i]*step;}}}
pub fn keycode(s:&str,fallback:bevy::prelude::KeyCode)->bevy::prelude::KeyCode{use bevy::prelude::KeyCode::*;match s.to_ascii_uppercase().as_str(){"A"=>KeyA,"D"=>KeyD,"W"=>KeyW,"S"=>KeyS,"LEFT"=>ArrowLeft,"RIGHT"=>ArrowRight,"UP"=>ArrowUp,"DOWN"=>ArrowDown,"SPACE"=>Space,"J"=>KeyJ,"L"=>KeyL,"I"=>KeyI,"K"=>KeyK,_=>fallback}}
#[cfg(test)] mod tests{use super::*;#[test]fn detour(){let p=pathfind([0,0],[2,0],&HashSet::from([[1,0]]),100).unwrap();assert_eq!(p.len(),5);assert!(!p.contains(&[1,0]));}#[test]fn bounded(){assert!(pathfind([0,0],[100,100],&HashSet::new(),10).is_none());}#[test]fn chunks(){let mut p=Project::default();crate::paint_tile(&mut p,[-1,0],32,0,None,false);assert!(tile_chunks(&p).contains_key(&(0,32,-1,0)));}}
