//! Portable gameplay utilities; shared by editor and standalone player.
use crate::{Body,Object,Project};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct PhysicsMaterial {pub gravity_scale:f32,pub speed:f32,pub jump_speed:f32,pub restitution:f32}
impl Default for PhysicsMaterial {fn default()->Self{Self{gravity_scale:1.,speed:200.,jump_speed:320.,restitution:0.}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct TileCell {pub layer:i32,pub cell:[i32;2],pub grid:u32}
/// Tiles are ordinary scene entities, so legacy renderers, physics and undo support them.
pub fn paint_tile(p:&mut Project,cell:[i32;2],grid:u32,layer:i32,sprite:Option<String>,solid:bool)->Uuid{
 let grid=grid.clamp(1,512); let tag=TileCell{layer,cell,grid};
 if let Some(o)=p.objects.iter_mut().find(|o|o.tile.as_ref()==Some(&tag)){o.sprite=sprite;o.body=if solid{Body::Static}else{Body::None};return o.id;}
 let mut o=Object::new("Tile",[cell[0] as f32*grid as f32,cell[1] as f32*grid as f32],[grid as f32;2],[255,255,255,255]);o.layer=layer;o.sprite=sprite;o.body=if solid{Body::Static}else{Body::None};o.tile=Some(tag);let id=o.id;p.objects.push(o);id
}
pub fn erase_tile(p:&mut Project,cell:[i32;2],grid:u32,layer:i32){p.objects.retain(|o|o.tile.as_ref()!=Some(&TileCell{layer,cell,grid}));}
/// Segment vs axis-aligned static colliders; returns closest hit in world units.
pub fn raycast(p:&Project,origin:[f32;2],direction:[f32;2],distance:f32)->Option<(Uuid,f32)>{
 let len=direction[0].hypot(direction[1]);if !len.is_finite()||len<1e-6||!distance.is_finite()||distance<0.{return None;}let d=direction.map(|v|v/len);
 p.objects.iter().filter(|o|o.body==Body::Static).filter_map(|o|{let mut near=0_f32;let mut far=distance;for i in 0..2{let lo=o.position[i]-o.size[i]*0.5;let hi=o.position[i]+o.size[i]*0.5;if d[i].abs()<1e-6{if origin[i]<lo||origin[i]>hi{return None;}}else{let a=(lo-origin[i])/d[i];let b=(hi-origin[i])/d[i];near=near.max(a.min(b));far=far.min(a.max(b));if near>far{return None;}}}Some((o.id,near))}).min_by(|a,b|a.1.total_cmp(&b.1))
}
#[cfg(test)] mod tests{use super::*;#[test]fn paint_replace_erase(){let mut p=Project::default();let n=p.objects.len();let a=paint_tile(&mut p,[4,5],32,1,None,true);assert_eq!(a,paint_tile(&mut p,[4,5],32,1,None,false));assert_eq!(p.objects.len(),n+1);erase_tile(&mut p,[4,5],32,1);assert_eq!(p.objects.len(),n);}#[test]fn ray_hit(){let mut p=Project::default();p.objects.clear();let mut o=Object::new("wall",[10.,0.],[2.,2.],[255;4]);o.body=Body::Static;p.objects.push(o);assert_eq!(raycast(&p,[0.,0.],[1.,0.],20.).unwrap().1,9.);assert!(raycast(&p,[0.,3.],[1.,0.],20.).is_none());}}

#[cfg(test)] mod graph_tests {use crate::{Graph,NodeKind};#[test]fn condition_gate(){let g=Graph{nodes:vec![NodeKind::Update,NodeKind::IfRight,NodeKind::MoveRight],edges:vec![[0,1],[1,2]]};assert!(!g.actions_input(NodeKind::Update,[-1.,0.]).contains(&NodeKind::MoveRight));assert!(g.actions_input(NodeKind::Update,[1.,0.]).contains(&NodeKind::MoveRight));}}
