//! Explicit sprite-clip transitions and timing. No rig generation or automatic animation authoring.
use crate::{Animation,Object};
use serde::{Deserialize,Serialize};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Serialize,Deserialize)]
pub enum ClipCondition { #[default] Idle, Moving, JumpInput, Finished, VariablePositive, VariableNonPositive }
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct ClipTransition { pub from:String,pub to:String,pub condition:ClipCondition,pub variable:String }
pub fn clip_frame_count(a:&Animation)->u32 {let total=a.total_frames();a.frames.max(1).min(total-a.start.min(total-1))}
pub fn frame_start(a:&Animation,index:u32)->f32 {let n=clip_frame_count(a);let i=index.min(n);if a.durations.len()==n as usize&&a.durations.iter().all(|&d|d>0){a.durations.iter().take(i as usize).map(|&d|u64::from(d)).sum::<u64>() as f32/1000.}else{i as f32/if a.fps.is_finite()&&a.fps>0.{a.fps}else{1.}}}
pub fn step_frame(a:&Animation,t:f32,delta:i32)->f32 {let n=clip_frame_count(a);let start=a.start.min(a.total_frames()-1);let i=i64::from(a.frame(t).saturating_sub(start))+i64::from(delta);frame_start(a,i.clamp(0,i64::from(n-1)) as u32)}
pub fn set_frame_duration(a:&mut Animation,i:u32,ms:u32)->bool {let n=clip_frame_count(a);if n>16384||i>=n{return false;}if a.durations.len()!=n as usize||a.durations.contains(&0){a.durations=vec![(1000./if a.fps.is_finite()&&a.fps>0.{a.fps}else{1.}).round().clamp(1.,60000.) as u32;n as usize];}a.durations[i as usize]=ms.clamp(1,60000);true}
pub fn transition_clip(o:&mut Object,input:[f32;2],elapsed:f32)->bool {
 let current=o.clip_name.as_str();let age=crate::animation_time(o,elapsed);let moving=input[0].abs()>0.01||input[1].abs()>0.01;
 let target=o.motion.transitions.iter().find(|r| (r.from==current||r.from=="*")&&r.to!=current&&o.clips.contains_key(&r.to)&&match r.condition{ClipCondition::Idle=>!moving,ClipCondition::Moving=>moving,ClipCondition::JumpInput=>input[1]>0.,ClipCondition::Finished=>!o.animation.looping&&age>=frame_start(&o.animation,clip_frame_count(&o.animation)),ClipCondition::VariablePositive=>!r.variable.is_empty()&&o.pro.variables.get(&r.variable).is_some_and(|v|*v>0.),ClipCondition::VariableNonPositive=>!r.variable.is_empty()&&o.pro.variables.get(&r.variable).copied().unwrap_or(0.)<=0.}).map(|r|r.to.clone());
 if let Some(target)=target {o.animation=o.clips[&target].clone();o.clip_name=target;o.pro.variables.insert("forge_clip_started".into(),elapsed);true}else{false}
}
#[cfg(test)]mod tests {use super::*;#[test]fn variable_timing(){let mut a=Animation{columns:3,frames:3,..Default::default()};assert!(set_frame_duration(&mut a,0,200));assert!(set_frame_duration(&mut a,1,500));assert!((frame_start(&a,2)-0.7).abs()<0.001);assert!((step_frame(&a,0.21,1)-0.7).abs()<0.001);}
#[test]fn explicit_transition(){let mut o=Object::new("actor",[0.;2],[16.;2],[255;4]);o.clips.insert("Run".into(),Animation::default());o.motion.transitions.push(ClipTransition{from:"Idle".into(),to:"Run".into(),condition:ClipCondition::Moving,variable:String::new()});assert!(!transition_clip(&mut o,[0.;2],1.));assert!(transition_clip(&mut o,[1.,0.],2.));assert_eq!(o.clip_name,"Run");assert_eq!(crate::animation_time(&o,2.),0.);}
#[test]fn legacy_defaults(){let m:crate::Motion=serde_json::from_str("{}").unwrap();assert!(m.transitions.is_empty());}}

#[cfg(test)]mod fixture_test{use super::*;#[test]fn studio_fixture(){let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/studio_pro/project.forge.json");let mut p=crate::Project::load(&path).unwrap();let o=p.objects.iter_mut().find(|o|o.player).unwrap();assert!(transition_clip(o,[1.,0.],1.));assert_eq!(o.clip_name,"Run");assert!(transition_clip(o,[0.,0.],2.));assert_eq!(o.clip_name,"Idle");assert!(p.settings.widgets.iter().any(|w|w.action=="progress"));}}

/// Sample inside a frame, avoiding floating-point rounding on a frame boundary.
pub fn frame_sample_time(a:&Animation,index:u32)->f32{let i=index.min(clip_frame_count(a)-1);let start=frame_start(a,i);start+(frame_start(a,i+1)-start)*0.5}
#[cfg(test)]mod sample_tests{use super::*;#[test]fn sample_each_variable_frame(){let a=Animation{columns:4,frames:3,start:1,durations:vec![123,251,73],..Default::default()};for i in 0..3{assert_eq!(a.frame(frame_sample_time(&a,i)),i+1);}}}

#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct AnimationEvent{pub clip:String,pub time:f32,pub name:String}
/// Crossed event markers. Handles looping and skipped frames; initial markers include t=0.
pub fn crossed_events(o:&Object,previous:f32,now:f32,restarted:bool)->Vec<String>{
 if !now.is_finite()||!previous.is_finite()||now<0.||(!restarted&&now<=previous){return Vec::new();}
 let duration=frame_start(&o.animation,clip_frame_count(&o.animation));if !duration.is_finite()||duration<=0.{return Vec::new();}
 let from=if restarted{-0.000001}else{previous.max(0.)};let mut result=Vec::new();for e in o.motion.events.iter().take(256){if e.clip!=o.clip_name||e.name.trim().is_empty()||e.name.len()>128||!e.time.is_finite()||e.time<0.||e.time>=duration{continue;}
 let first=if o.animation.looping{(((from-e.time)/duration).floor() as i64).saturating_add(1).max(0)}else{0};let last=if o.animation.looping{(((now-e.time)/duration).floor() as i64).max(-1)}else{0};for k in first..=last.min(first.saturating_add(15)){let t=e.time+k as f32*duration;if t>from&&t<=now{result.push((t,e.name.clone()));}}}
 result.sort_by(|a,b|a.0.total_cmp(&b.0));result.into_iter().take(64).map(|(_,name)|name).collect()
}
#[cfg(test)]mod event_tests{use super::*;#[test]fn loops_and_start(){let mut o=Object::new("a",[0.;2],[1.;2],[255;4]);o.animation.fps=1.;o.motion.events=vec![AnimationEvent{clip:o.clip_name.clone(),time:0.,name:"start".into()},AnimationEvent{clip:o.clip_name.clone(),time:0.5,name:"hit".into()}];assert_eq!(crossed_events(&o,0.,0.1,true),vec!["start"]);assert_eq!(crossed_events(&o,0.4,1.1,false),vec!["hit","start"]);o.animation.looping=false;assert!(crossed_events(&o,1.1,2.1,false).is_empty());}}

#[cfg(test)]mod event_runtime_tests{use super::*;#[test]fn simulation_emits_once(){let mut p=crate::Project::default();let o=p.objects.iter_mut().find(|o|o.player).unwrap();o.animation.fps=1.;o.motion.events.push(AnimationEvent{clip:o.clip_name.clone(),time:0.02,name:"cue".into()});let mut sim=crate::Simulation::default();for _ in 0..3{sim.step(&mut p,0.03,[0.;2]);}assert_eq!(sim.events.iter().filter(|s|s.ends_with(": cue")).count(),1);}}

#[cfg(test)]mod slow35{use super::*;#[test]fn half_fps_matches_frame(){let a=Animation{columns:2,frames:2,fps:0.5,..Default::default()};assert_eq!(frame_start(&a,1),2.);assert_eq!(a.frame(frame_sample_time(&a,1)),1);}}
