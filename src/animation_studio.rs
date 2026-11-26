//! Explicit sprite-clip transitions and timing. No rig generation or automatic animation authoring.
use crate::{Animation,Object};
use serde::{Deserialize,Serialize};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Serialize,Deserialize)]
pub enum ClipCondition { #[default] Idle, Moving, JumpInput, Finished, VariablePositive }
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct ClipTransition { pub from:String,pub to:String,pub condition:ClipCondition,pub variable:String }
pub fn clip_frame_count(a:&Animation)->u32 {let total=a.columns.max(1).saturating_mul(a.rows.max(1));a.frames.max(1).min(total-a.start.min(total-1))}
pub fn frame_start(a:&Animation,index:u32)->f32 {let n=clip_frame_count(a);let i=index.min(n);if a.durations.len()==n as usize&&a.durations.iter().all(|&d|d>0){a.durations.iter().take(i as usize).map(|&d|u64::from(d)).sum::<u64>() as f32/1000.}else{i as f32/a.fps.max(1.)}}
pub fn step_frame(a:&Animation,t:f32,delta:i32)->f32 {let n=clip_frame_count(a);let start=a.start.min(a.columns.max(1).saturating_mul(a.rows.max(1))-1);let i=i64::from(a.frame(t).saturating_sub(start))+i64::from(delta);frame_start(a,i.clamp(0,i64::from(n-1)) as u32)}
pub fn set_frame_duration(a:&mut Animation,i:u32,ms:u32)->bool {let n=clip_frame_count(a);if n>16384||i>=n{return false;}if a.durations.len()!=n as usize||a.durations.contains(&0){a.durations=vec![(1000./a.fps.max(1.)).round().max(1.) as u32;n as usize];}a.durations[i as usize]=ms.clamp(1,60000);true}
pub fn transition_clip(o:&mut Object,input:[f32;2],elapsed:f32)->bool {
 let current=o.clip_name.as_str();let age=crate::animation_time(o,elapsed);let moving=input[0].abs()>0.01||input[1].abs()>0.01;
 let target=o.motion.transitions.iter().find(|r| (r.from==current||r.from=="*")&&r.to!=current&&o.clips.contains_key(&r.to)&&match r.condition{ClipCondition::Idle=>!moving,ClipCondition::Moving=>moving,ClipCondition::JumpInput=>input[1]>0.,ClipCondition::Finished=>!o.animation.looping&&age>=frame_start(&o.animation,clip_frame_count(&o.animation)),ClipCondition::VariablePositive=>!r.variable.is_empty()&&o.pro.variables.get(&r.variable).is_some_and(|v|*v>0.)}).map(|r|r.to.clone());
 if let Some(target)=target {o.animation=o.clips[&target].clone();o.clip_name=target;o.pro.variables.insert("forge_clip_started".into(),elapsed);true}else{false}
}
#[cfg(test)]mod tests {use super::*;#[test]fn variable_timing(){let mut a=Animation{columns:3,frames:3,..Default::default()};assert!(set_frame_duration(&mut a,0,200));assert!(set_frame_duration(&mut a,1,500));assert!((frame_start(&a,2)-0.7).abs()<0.001);assert!((step_frame(&a,0.21,1)-0.7).abs()<0.001);}
#[test]fn explicit_transition(){let mut o=Object::new("actor",[0.;2],[16.;2],[255;4]);o.clips.insert("Run".into(),Animation::default());o.motion.transitions.push(ClipTransition{from:"Idle".into(),to:"Run".into(),condition:ClipCondition::Moving,variable:String::new()});assert!(!transition_clip(&mut o,[0.;2],1.));assert!(transition_clip(&mut o,[1.,0.],2.));assert_eq!(o.clip_name,"Run");assert_eq!(crate::animation_time(&o,2.),0.);}
#[test]fn legacy_defaults(){let m:crate::Motion=serde_json::from_str("{}").unwrap();assert!(m.transitions.is_empty());}}

#[cfg(test)]mod fixture_test{use super::*;#[test]fn studio_fixture(){let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/studio_pro/project.forge.json");let mut p=crate::Project::load(&path).unwrap();let o=p.objects.iter_mut().find(|o|o.player).unwrap();assert!(transition_clip(o,[1.,0.],1.));assert_eq!(o.clip_name,"Run");assert!(transition_clip(o,[0.,0.],2.));assert_eq!(o.clip_name,"Idle");assert!(p.settings.widgets.iter().any(|w|w.action=="progress"));}}
