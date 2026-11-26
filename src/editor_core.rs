//! World-space hierarchy editing, shared by editor and player.
use crate::{Object,Project};
use uuid::Uuid;
use std::collections::{HashMap,HashSet};

pub fn reparent(p:&mut Project,child:Uuid,parent:Option<Uuid>)->Result<(),String>{
    let mut cursor=parent; let mut seen=HashSet::from([child]);
    while let Some(id)=cursor { if !seen.insert(id){return Err("Hierarchy cycle".into());} cursor=p.objects.iter().find(|o|o.id==id).ok_or("Missing parent")?.parent; }
    p.objects.iter_mut().find(|o|o.id==child).ok_or("Missing child")?.parent=parent; Ok(())
}
fn rotate(p:[f32;2],degrees:f32)->[f32;2]{let(s,c)=degrees.to_radians().sin_cos();[p[0]*c-p[1]*s,p[0]*s+p[1]*c]}
/// Propagate parent movement/rotation to descendants whose own transform was not edited.
/// Existing files retain world coordinates; reparenting preserves their appearance.
pub fn inherit_world(before:&Project,after:&mut Project){
 let old:HashMap<_,_>=before.objects.iter().map(|o|(o.id,o)).collect();
 let mut done=HashSet::new(); let mut pending:HashSet<_>=after.objects.iter().map(|o|o.id).collect();
 while !pending.is_empty(){let n=pending.len();for i in 0..after.objects.len(){let id=after.objects[i].id;if !pending.contains(&id){continue}let parent=after.objects[i].parent;if parent.is_some_and(|p|pending.contains(&p)&&!done.contains(&p)){continue}
 if let (Some(prev),Some(pid))=(old.get(&id),parent){if prev.parent==parent {if let (Some(a),Some(b))=(old.get(&pid),after.objects.iter().find(|o|o.id==pid).cloned()){let o=&mut after.objects[i];if o.position==prev.position&&o.rotation==prev.rotation{let d=rotate([o.position[0]-a.position[0],o.position[1]-a.position[1]],b.rotation-a.rotation);o.position=[b.position[0]+d[0],b.position[1]+d[1]];o.rotation+=b.rotation-a.rotation;}}}}
 pending.remove(&id);done.insert(id);}if pending.len()==n{break;}}
}
pub fn sprite_center(o:&Object,iso:bool)->[f32;2]{let p=crate::project_point(o.position,iso);let d=rotate([-o.motion.pivot[0]*o.size[0],-o.motion.pivot[1]*o.size[1]],o.rotation);[p[0]+d[0],p[1]+d[1]]}
#[cfg(test)]mod tests{use super::*;#[test]fn parent_delta(){let mut p=Project::default();let id=p.objects[0].id;let child=p.objects[1].id;reparent(&mut p,child,Some(id)).unwrap();let old=p.clone();p.objects[0].position[0]+=20.;inherit_world(&old,&mut p);assert_eq!(p.objects[1].position[0],old.objects[1].position[0]+20.);assert!(reparent(&mut p,id,Some(child)).is_err());}#[test]fn pivot(){let mut o=Object::new("test",[0.,0.],[20.,40.],[255;4]);o.motion.pivot=[0.5,0.];assert_eq!(sprite_center(&o,false),[-10.,0.]);}}

/// Visual bounds of an animation frame, independent of logical object/physics bounds.
pub fn sprite_frame_geometry(o:&Object,iso:bool,time:f32)->([f32;2],[f32;2]) {
 let visual=sprite_visual_channels(o,time);let size=[o.size[0]*visual[0],o.size[1]*visual[1]];let p=crate::project_point(o.position,iso);let pivot=rotate([-o.motion.pivot[0]*size[0],-o.motion.pivot[1]*size[1]],o.rotation);let center=[p[0]+pivot[0],p[1]+pivot[1]];
 let frame=o.animation.frame(time);
 let Some(&g)=o.animation.frame_geometry.get(&frame).filter(|g|crate::valid_region(**g)&&o.sprite.is_some()) else{return (center,size);};
 let offset=rotate([(g[0]+g[2]*0.5-0.5)*size[0],(0.5-g[1]-g[3]*0.5)*size[1]],o.rotation);
 ([center[0]+offset[0],center[1]+offset[1]],[size[0]*g[2],size[1]*g[3]])
}
#[cfg(test)]mod frame_geometry_tests{use super::*;#[test]fn trim_keeps_world_pixels_and_collider(){let mut o=Object::new("trim",[10.,20.],[100.,80.],[255;4]);o.sprite=Some("assets/a.png".into());o.motion.pivot=[0.1,-0.25];o.animation.frame_geometry.insert(0,[0.25,0.125,0.5,0.5]);let (c,s)=sprite_frame_geometry(&o,false,0.);assert_eq!(c,[0.,50.]);assert_eq!(s,[50.,40.]);assert_eq!(o.size,[100.,80.]);o.rotation=90.;let (c,_)=sprite_frame_geometry(&o,false,0.);assert!((c[0]+20.).abs()<0.001&&(c[1]-10.).abs()<0.001);}}

/// Clip-local visual scale X/Y and opacity multiplier. Never mutates logical object or collider.
pub fn sprite_visual_channels(o:&Object,age:f32)->[f32;3]{let Some(ch)=o.motion.visual_channels.get(&o.clip_name)else{return [1.;3];};let duration=crate::frame_start(&o.animation,crate::clip_frame_count(&o.animation));let t=if o.animation.looping&&duration.is_finite()&&duration>0.&&age.is_finite(){age.max(0.).rem_euclid(duration)}else{age};std::array::from_fn(|i|crate::sample_scalar(&ch[i],t).unwrap_or(1.).clamp(if i==2{0.}else{0.01},if i==2{1.}else{16.}))}
pub fn sprite_visual_alpha(o:&Object,age:f32,alpha:u8)->u8{(alpha as f32*sprite_visual_channels(o,age)[2]).round() as u8}
#[cfg(test)]mod visual_channel_tests{use super::*;#[test]fn geometry_and_alpha_do_not_change_body(){let mut o=Object::new("a",[10.,20.],[20.,40.],[255;4]);o.motion.pivot=[0.5,0.];o.motion.visual_channels.insert(o.clip_name.clone(),[vec![crate::ScalarKey{time:0.,value:2.,smooth:false,bezier:None,bezier_x:None}],vec![],vec![crate::ScalarKey{time:0.,value:0.5,smooth:false,bezier:None,bezier_x:None}]]);assert_eq!(sprite_frame_geometry(&o,false,0.),([-10.,20.],[40.,40.]));assert_eq!(sprite_visual_alpha(&o,0.,200),100);assert_eq!(o.size,[20.,40.]);assert_eq!(o.color,[255;4]);}}
