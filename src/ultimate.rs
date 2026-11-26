use crate::{Object,Project};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Motion { #[serde(default,skip_serializing_if="std::collections::BTreeMap::is_empty")] pub visual_channels:std::collections::BTreeMap<String,[Vec<ScalarKey>;3]>, #[serde(default,skip_serializing_if="std::collections::BTreeMap::is_empty")] pub property_channels:std::collections::BTreeMap<String,[Vec<ScalarKey>;3]>, #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")] pub clip_keys:std::collections::BTreeMap<String,Vec<Key>>, pub events:Vec<crate::AnimationEvent>,pub keys:Vec<Key>,pub pivot:[f32;2],pub hitboxes:std::collections::BTreeMap<u32,[f32;4]>,pub auto_state:bool, pub transitions:Vec<crate::ClipTransition>, pub graph_positions:std::collections::BTreeMap<String,[f32;2]> }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Key {pub time:f32,pub position:[f32;2],pub rotation:f32,pub smooth:bool,#[serde(default)]pub bezier:Option<[[f32;2];3]>}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct ScalarKey{pub time:f32,pub value:f32,#[serde(default)]pub smooth:bool,#[serde(default,skip_serializing_if="Option::is_none")]pub bezier:Option<[f32;2]>,#[serde(default,skip_serializing_if="Option::is_none")]pub bezier_x:Option<[f32;2]>}
/// Invert monotonic Bezier x(t) to evaluate y at the requested normalized time.
pub fn bezier_ease(x:f32,handles_x:[f32;2],handles_y:[f32;2])->f32{
 let x=x.clamp(0.,1.);let cubic=|t:f32,a:f32,b:f32|{let v=1.-t;3.*v*v*t*a+3.*v*t*t*b+t*t*t};
 if x==0.||x==1.{return x;}let [a,b]=handles_x.map(|v|if v.is_finite(){v.clamp(0.,1.)}else{0.5});
 let [c,d]=handles_y.map(|v|if v.is_finite(){v.clamp(-2.,3.)}else{0.5});
 if [a,b]==[1./3.,2./3.]{return cubic(x,c,d);}let(mut lo,mut hi)=(0.,1.);for _ in 0..24{let t=(lo+hi)*0.5;if cubic(t,a,b)<x{lo=t;}else{hi=t;}}cubic((lo+hi)*0.5,c,d)
}
pub fn sample_scalar(keys:&[ScalarKey],t:f32)->Option<f32>{if !t.is_finite(){return None;}let mut lo:Option<&ScalarKey>=None;let mut hi:Option<&ScalarKey>=None;for k in keys.iter().filter(|k|k.time.is_finite()&&k.value.is_finite()){if k.time<t{if lo.is_none_or(|v|k.time>=v.time){lo=Some(k);}}else if hi.is_none_or(|v|k.time<v.time){hi=Some(k);}}match(lo,hi){(Some(a),Some(b))=>{let mut u=((t-a.time)/(b.time-a.time).max(0.00001)).clamp(0.,1.);if let Some([c1,c2])=a.bezier{u=bezier_ease(u,a.bezier_x.unwrap_or([1./3.,2./3.]),[c1,c2]);}else if a.smooth{u=u*u*(3.-2.*u);}Some(a.value+(b.value-a.value)*u)},(Some(a),None)|(None,Some(a))=>Some(a.value),(None,None)=>None}}
impl Motion {pub fn sample(&self,t:f32)->Option<([f32;2],f32)>{Self::sample_keys(&self.keys,t)}
 pub fn sample_clip(&self,clip:&str,age:f32,elapsed:f32,animation:&crate::Animation)->Option<([f32;2],f32)>{if !self.clip_keys.contains_key(clip)&&!self.property_channels.contains_key(clip){return self.sample(elapsed);}let duration=crate::frame_start(animation,crate::clip_frame_count(animation));let t=if animation.looping&&duration.is_finite()&&duration>0.&&age.is_finite(){age.max(0.).rem_euclid(duration)}else{age};let base=match self.clip_keys.get(clip){None=>self.sample(elapsed),Some(keys)=>Self::sample_keys(keys,t)};let (mut position,mut rotation)=base?;if let Some(ch)=self.property_channels.get(clip){for i in 0..2{if let Some(v)=sample_scalar(&ch[i],t){position[i]=v;}}if let Some(v)=sample_scalar(&ch[2],t){rotation=v;}}Some((position,rotation))}
 pub fn sample_clip_with_base(&self,clip:&str,age:f32,elapsed:f32,animation:&crate::Animation,base:([f32;2],f32))->Option<([f32;2],f32)>{
  if let Some(v)=self.sample_clip(clip,age,elapsed,animation){return Some(v);}
  let ch=self.property_channels.get(clip)?;if ch.iter().all(Vec::is_empty){return None;}
  let duration=crate::frame_start(animation,crate::clip_frame_count(animation));let t=if animation.looping&&duration.is_finite()&&duration>0.&&age.is_finite(){age.max(0.).rem_euclid(duration)}else{age};
  let(mut pos,mut rot)=base;for i in 0..2{if let Some(v)=sample_scalar(&ch[i],t){pos[i]=v;}}if let Some(v)=sample_scalar(&ch[2],t){rot=v;}Some((pos,rot))
 }
 pub fn valid_clip_keys(&self)->bool{self.visual_channels.len()<=128&&self.visual_channels.iter().all(|(name,ch)|!name.is_empty()&&name.len()<=128&&ch.iter().enumerate().all(|(axis,keys)|keys.len()<=1024&&keys.iter().all(|k|k.time.is_finite()&&k.time>=0.&&k.time<=3600.&&k.value.is_finite()&&(if axis==2{(0. ..=1.).contains(&k.value)}else{(0.01..=16.).contains(&k.value)})&&k.bezier_x.is_none_or(|b|b.iter().all(|v|v.is_finite()&&(0. ..=1.).contains(v)))&&k.bezier.is_none_or(|b|b.iter().all(|v|v.is_finite()&&(-2. ..=3.).contains(v))))))&&self.property_channels.len()<=128&&self.property_channels.iter().all(|(name,ch)|!name.is_empty()&&name.len()<=128&&ch.iter().all(|keys|keys.len()<=1024&&keys.iter().all(|k|k.time.is_finite()&&k.time>=0.&&k.time<=3600.&&k.value.is_finite()&&k.bezier_x.is_none_or(|b|b.iter().all(|v|v.is_finite()&&(0. ..=1.).contains(v)))&&k.bezier.is_none_or(|b|b.iter().all(|v|v.is_finite()&&(-2. ..=3.).contains(v))))))&&self.clip_keys.len()<=128&&self.clip_keys.iter().all(|(name,keys)|!name.is_empty()&&name.len()<=128&&keys.len()<=1024&&keys.iter().all(|k|k.time.is_finite()&&k.time>=0.&&k.time<=3600.&&k.position.iter().chain([&k.rotation]).all(|v|v.is_finite())&&k.bezier.as_ref().is_none_or(|b|b.iter().flatten().all(|v|v.is_finite()&&*v>=0.&&*v<=1.))))}
 fn sample_keys(keys:&[Key],t:f32)->Option<([f32;2],f32)>{if !t.is_finite(){return None;}let mut lo:Option<&Key>=None;let mut hi:Option<&Key>=None;for k in keys.iter().filter(|k|k.time.is_finite()){if k.time<t{if lo.is_none_or(|v|k.time>=v.time){lo=Some(k);}}else if hi.is_none_or(|v|k.time<v.time){hi=Some(k);}}let (a,b)=match(lo,hi){(Some(a),Some(b))=>(a,b),(Some(a),None)|(None,Some(a))=>return Some((a.position,a.rotation)),(None,None)=>return None};let mut u=((t-a.time)/(b.time-a.time).max(0.00001)).clamp(0.,1.);if a.smooth{u=u*u*(3.-2.*u);}let raw=((t-a.time)/(b.time-a.time).max(0.00001)).clamp(0.,1.);let blend=|i:usize|{if let Some(handles)=a.bezier{let [c1,c2]=handles[i];let v=1.-raw;3.*v*v*raw*c1+3.*v*raw*raw*c2+raw*raw*raw}else{u}};return Some((std::array::from_fn(|i|a.position[i]+(b.position[i]-a.position[i])*blend(i)),a.rotation+(b.rotation-a.rotation)*blend(2)));}}
pub fn prefab(p:&Project,id:Uuid)->Vec<Object>{let mut ids=std::collections::HashSet::from([id]);loop{let n=ids.len();for o in &p.objects{if o.parent.is_some_and(|x|ids.contains(&x)){ids.insert(o.id);}}if ids.len()==n{break;}}p.objects.iter().filter(|o|ids.contains(&o.id)).cloned().map(|mut o|{if o.id==id{o.parent=None;}o}).collect()}
pub fn instantiate(mut objects:Vec<Object>)->Vec<Object>{let map:std::collections::HashMap<_,_>=objects.iter().map(|o|(o.id,Uuid::new_v4())).collect();for o in &mut objects{o.prefab_link=None;o.id=map[&o.id];o.parent=o.parent.and_then(|p|map.get(&p).copied());}objects}
#[cfg(test)]mod tests{use super::*;#[test]fn interpolation(){let m=Motion{keys:vec![Key{time:0.,position:[0.,0.],rotation:0.,smooth:false,bezier:None},Key{time:2.,position:[10.,20.],rotation:90.,smooth:false,bezier:None}],..Default::default()};assert_eq!(m.sample(1.),Some(([5.,10.],45.)));}#[test]fn prefab_ids(){let p=Project::default();let a=instantiate(prefab(&p,p.objects[0].id));assert_ne!(a[0].id,p.objects[0].id);}}

#[cfg(test)]mod sampling_regression{use super::*;#[test]fn unsorted_and_duplicate_keys(){let key=|time,position|Key{time,position:[position,0.],rotation:0.,smooth:false,bezier:None};let m=Motion{keys:vec![key(2.,20.),key(0.,0.),key(1.,10.),key(1.,15.)],..Default::default()};assert_eq!(m.sample(0.5).unwrap().0,[5.,0.]);assert_eq!(m.sample(1.).unwrap().0,[10.,0.]);assert_eq!(m.sample(1.5).unwrap().0,[17.5,0.]);assert_eq!(m.sample(5.).unwrap().0,[20.,0.]);}}

#[cfg(test)]mod channel_tests{use super::*;#[test]fn clip_local_loop_and_legacy(){let key=|time,x|Key{time,position:[x,0.],rotation:0.,smooth:false,bezier:None};let mut m=Motion::default();m.keys=vec![key(0.,50.),key(2.,70.)];m.clip_keys.insert("Run".into(),vec![key(0.,0.),key(1.,10.)]);m.clip_keys.insert("Empty".into(),vec![]);let a=crate::Animation{frames:2,columns:2,fps:1.,..Default::default()};assert_eq!(m.sample_clip("Run",2.5,10.,&a).unwrap().0,[5.,0.]);assert_eq!(m.sample_clip("Idle",0.,1.,&a).unwrap().0,[60.,0.]);assert!(m.sample_clip("Empty",0.,1.,&a).is_none());assert!(m.valid_clip_keys());assert!(serde_json::from_str::<Motion>("{}").unwrap().clip_keys.is_empty());}}

#[cfg(test)]mod property_tests{use super::*;#[test]fn independent_axes_and_legacy(){let mut m=Motion::default();m.keys.push(Key{time:0.,position:[4.,8.],rotation:90.,smooth:false,bezier:None});m.property_channels.insert("Run".into(),[vec![ScalarKey{time:0.,value:0.,smooth:false,bezier:None,bezier_x:None},ScalarKey{time:2.,value:20.,smooth:false,bezier:None,bezier_x:None}],vec![],vec![ScalarKey{time:0.,value:15.,smooth:false,bezier:None,bezier_x:None}]]);let a=crate::Animation{looping:false,..Default::default()};assert_eq!(m.sample_clip("Run",1.,1.,&a),Some(([10.,8.],15.)));assert_eq!(m.sample_clip("Idle",1.,1.,&a),Some(([4.,8.],90.)));assert!(m.valid_clip_keys());assert!(serde_json::from_str::<Motion>("{}").unwrap().property_channels.is_empty());}}

#[cfg(test)]mod bezier28{use super::*;#[test]fn old_keys_and_cubic(){let mut a:ScalarKey=serde_json::from_str(r#"{"time":0,"value":0,"smooth":false}"#).unwrap();assert!(a.bezier.is_none());a.bezier=Some([0.,0.]);let b=ScalarKey{time:1.,value:8.,smooth:false,bezier:None,bezier_x:None};assert_eq!(sample_scalar(&[a,b],0.5),Some(1.));}}

/// Paste a relative-time group transactionally, rejecting ambiguous time collisions.
pub fn paste_scalar_group(keys:&mut Vec<ScalarKey>,clip:&[ScalarKey],at:f32)->Result<Vec<usize>,String>{
 if clip.is_empty()||keys.len()+clip.len()>1024||!at.is_finite(){return Err("Invalid clipboard or key budget".into());}
 let first=clip.iter().map(|k|k.time).fold(f32::INFINITY,f32::min);let mut pending=Vec::with_capacity(clip.len());
 for k in clip{let mut k=k.clone();k.time=at+(k.time-first);if !k.time.is_finite()||!(0. ..=3600.).contains(&k.time)||!k.value.is_finite()||keys.iter().chain(pending.iter()).any(|old:&ScalarKey|(old.time-k.time).abs()<0.0001){return Err("Paste overlaps an existing key or exceeds timeline".into());}pending.push(k);}
 let start=keys.len();keys.extend(pending);Ok((start..keys.len()).collect())
}
#[cfg(test)]mod group30{use super::*;#[test]fn paste_relative_and_atomic(){let k=|time|ScalarKey{time,value:3.,smooth:false,bezier:Some([0.2,0.8]),bezier_x:None};let mut keys=vec![k(0.)];let clip=vec![k(2.),k(3.)];assert_eq!(paste_scalar_group(&mut keys,&clip,4.).unwrap(),vec![1,2]);assert_eq!(keys[2].time,5.);let before=keys.clone();assert!(paste_scalar_group(&mut keys,&clip,4.).is_err());assert_eq!(keys,before);}}

/// Move selected keys as a transaction, preserving time intervals and avoiding collisions.
pub fn move_scalar_group(keys:&mut [ScalarKey],group:&[usize],delta:f32,limit:f32)->bool{
 if group.is_empty()||!delta.is_finite()||!limit.is_finite()||limit<=0.||group.iter().any(|&i|i>=keys.len()){return false;}
 for &i in group{let t=keys[i].time+delta;if !t.is_finite()||t<0.||t>limit||keys.iter().enumerate().any(|(j,k)|!group.contains(&j)&&(t-k.time).abs()<0.0001){return false;}}
 for (i,k) in keys.iter_mut().enumerate(){if group.contains(&i){k.time+=delta;}}true
}
#[cfg(test)]mod move31{use super::*;#[test]fn preserves_spacing_and_rejects_collision(){let k=|time|ScalarKey{time,value:1.,smooth:false,bezier:None,bezier_x:None};let mut keys=vec![k(0.),k(1.),k(3.)];assert!(move_scalar_group(&mut keys,&[0,1],1.,4.));assert_eq!(keys[1].time-keys[0].time,1.);let old=keys.clone();assert!(!move_scalar_group(&mut keys,&[0,1],1.,4.));assert_eq!(keys,old);}}

/// Shift selected time ranges across all enabled channels atomically; no partial changes.
pub fn move_multi_channel(m:&mut Motion,clip:&str,axes:[bool;6],range:[f32;2],delta:f32)->Result<usize,String>{
 if !range.iter().all(|v|v.is_finite())||!delta.is_finite(){return Err("Invalid range or offset".into());}let lo=range[0].min(range[1]);let hi=range[0].max(range[1]);
 let mut tracks=[m.property_channels.get(clip).cloned(),m.visual_channels.get(clip).cloned()];let mut count=0;
 for (bank,track) in tracks.iter_mut().enumerate(){if let Some(ch)=track{for (axis,keys) in ch.iter_mut().enumerate(){if !axes[bank*3+axis]{continue;}let group:Vec<_>=keys.iter().enumerate().filter(|(_,k)|k.time>=lo&&k.time<=hi).map(|(i,_)|i).collect();if group.is_empty(){continue;}if !move_scalar_group(keys,&group,delta,3600.){return Err("Group collision or timeline bounds; no channels changed".into());}count+=group.len();}}}
 if count==0{return Err("No keys in selected channels and time range".into());}if let Some(ch)=tracks[0].take(){m.property_channels.insert(clip.into(),ch);}if let Some(ch)=tracks[1].take(){m.visual_channels.insert(clip.into(),ch);}Ok(count)
}
#[cfg(test)]mod curve32{use super::*;#[test]fn horizontal_handles_change_timing(){assert!((bezier_ease(0.5,[0.,0.],[0.,1.])-0.89).abs()<0.04);assert_eq!(bezier_ease(0.,[1.,0.],[1.,0.]),0.);assert_eq!(bezier_ease(1.,[1.,0.],[1.,0.]),1.);}#[test]fn transactional_channels(){let k=|time|ScalarKey{time,value:1.,smooth:false,bezier:None,bezier_x:None};let mut m=Motion::default();m.property_channels.insert("Run".into(),[vec![k(1.)],vec![k(1.),k(2.)],vec![]]);let old=m.clone();assert!(move_multi_channel(&mut m,"Run",[true;6],[1.,1.],1.).is_err());assert_eq!(m,old);assert_eq!(move_multi_channel(&mut m,"Run",[true;6],[1.,1.],0.5).unwrap(),2);assert_eq!(m.property_channels["Run"][1][0].time,1.5);}}

#[cfg(test)]mod overshoot33{use super::*;#[test]fn overshoot_and_endpoints(){assert!(bezier_ease(0.8,[1./3.,2./3.],[0.4,2.])>1.);assert_eq!(bezier_ease(1.,[0.3,0.7],[-1.,2.]),1.);}#[test]fn standalone_property_track(){let mut m=Motion::default();m.property_channels.insert("Idle".into(),[vec![ScalarKey{time:0.,value:7.,smooth:false,bezier:None,bezier_x:None}],vec![],vec![]]);assert_eq!(m.sample_clip_with_base("Idle",0.,0.,&Default::default(),([1.,2.],3.)),Some(([7.,2.],3.)));}}

/// Clipboard times are relative to one shared origin across six lanes and events.
#[derive(Clone,Debug,Default,PartialEq)]
pub struct TimelineClipboard {pub tracks:[Vec<ScalarKey>;6],pub events:Vec<crate::AnimationEvent>}
impl TimelineClipboard {
 pub fn capture(m:&Motion,clip:&str,axes:[bool;6],range:[f32;2],events:bool)->Self{
  let mut result=Self::default();if !range.iter().all(|v|v.is_finite()){return result;}let lo=range[0].min(range[1]);let hi=range[0].max(range[1]);
  for i in 0..6{if !axes[i]{continue;}let bank=if i<3{&m.property_channels}else{&m.visual_channels};if let Some(ch)=bank.get(clip){result.tracks[i]=ch[i%3].iter().filter(|k|(lo..=hi).contains(&k.time)).cloned().collect();}}
  if events{result.events=m.events.iter().filter(|e|e.clip==clip&&(lo..=hi).contains(&e.time)).cloned().collect();}
  let origin=result.tracks.iter().flatten().map(|k|k.time).chain(result.events.iter().map(|e|e.time)).fold(f32::INFINITY,f32::min);
  for k in result.tracks.iter_mut().flatten(){k.time-=origin;}for e in &mut result.events{e.time-=origin;}result
 }
 pub fn count(&self)->usize{self.tracks.iter().map(Vec::len).sum::<usize>()+self.events.len()}
 pub fn paste(&self,m:&mut Motion,clip:&str,at:f32,event_end:f32)->Result<usize,String>{
  if self.count()==0||!at.is_finite()||!(0. ..=3600.).contains(&at)||clip.is_empty()||clip.len()>128{return Err("Invalid clipboard, clip or playhead".into());}
  let mut draft=m.clone();for i in 0..6{if self.tracks[i].is_empty(){continue;}let bank=if i<3{&mut draft.property_channels}else{&mut draft.visual_channels};let keys=&mut bank.entry(clip.to_owned()).or_default()[i%3];if keys.len()+self.tracks[i].len()>1024{return Err("Channel key budget exceeded".into());}for key in &self.tracks[i]{let mut k=key.clone();k.time+=at;if !k.time.is_finite()||!(0. ..=3600.).contains(&k.time)||keys.iter().any(|old|(old.time-k.time).abs()<0.0001){return Err("Key collision or bounds; nothing pasted".into());}keys.push(k);}}
  if draft.events.len()+self.events.len()>256{return Err("Event budget exceeded".into());}for event in &self.events{let mut e=event.clone();e.clip=clip.into();e.time+=at;if !event_end.is_finite()||!e.time.is_finite()||e.time<0.||e.time>=event_end||e.name.len()>128||draft.events.iter().any(|old|old.clip==e.clip&&old.name==e.name&&(old.time-e.time).abs()<0.0001){return Err("Event collision or outside clip; nothing pasted".into());}draft.events.push(e);}
  if !draft.valid_clip_keys(){return Err("Invalid channel data; nothing pasted".into());}*m=draft;Ok(self.count())
 }
}
#[cfg(test)]mod clipboard34{use super::*;#[test]fn shared_origin_and_rollback(){let key=|time|ScalarKey{time,value:1.,smooth:false,bezier:None,bezier_x:None};let mut m=Motion::default();m.property_channels.insert("A".into(),[vec![key(1.)],vec![key(2.)],vec![]]);m.events.push(crate::AnimationEvent{clip:"A".into(),time:1.5,name:"step".into()});let c=TimelineClipboard::capture(&m,"A",[true;6],[0.,3.],true);assert_eq!(c.count(),3);assert_eq!(c.paste(&mut m,"B",4.,6.).unwrap(),3);assert_eq!(m.property_channels["B"][1][0].time,5.);assert_eq!(m.events[1].time,4.5);let old=m.clone();assert!(c.paste(&mut m,"B",4.,6.).is_err());assert_eq!(m,old);assert!(c.paste(&mut m,"C",4.,4.1).is_err());assert_eq!(m,old);}}

/// Transactional shift across scalar lanes and the optional clip-event lane.
pub fn move_timeline_group(m:&mut Motion,clip:&str,axes:[bool;6],range:[f32;2],delta:f32,events:bool,duration:f32)->Result<usize,String>{
 if !delta.is_finite()||!range.iter().all(|v|v.is_finite())||!duration.is_finite()||duration<=0.{return Err("Invalid timeline interval".into());}
 let (lo,hi)=(range[0].min(range[1]),range[0].max(range[1]));let mut draft=m.clone();let captured=TimelineClipboard::capture(m,clip,axes,range,events);let mut count=0;
 if captured.tracks.iter().any(|k|!k.is_empty()){count+=move_multi_channel(&mut draft,clip,axes,range,delta)?;}
 if events{for e in &mut draft.events{if e.clip==clip&&(lo..=hi).contains(&e.time){e.time+=delta;if !(0. ..duration).contains(&e.time){return Err("Event outside clip duration".into());}count+=1;}}
  for (i,e) in draft.events.iter().enumerate().filter(|(_,e)|e.clip==clip){if draft.events[..i].iter().any(|v|v.clip==clip&&v.name==e.name&&(v.time-e.time).abs()<0.0001){return Err("Event collision; no lanes changed".into());}}
 }
 if count==0{return Err("No selected keys or events".into());}*m=draft;Ok(count)
}
pub fn delete_timeline_events(m:&mut Motion,clip:&str,range:[f32;2])->usize{if !range.iter().all(|v|v.is_finite()){return 0;}let old=m.events.len();let(lo,hi)=(range[0].min(range[1]),range[0].max(range[1]));m.events.retain(|e|e.clip!=clip||!(lo..=hi).contains(&e.time));old-m.events.len()}
#[cfg(test)]mod timeline35{use super::*;#[test]fn event_shift_atomic(){let mut m=Motion::default();m.events=vec![crate::AnimationEvent{clip:"A".into(),name:"step".into(),time:0.2},crate::AnimationEvent{clip:"A".into(),name:"step".into(),time:0.5}];let old=m.clone();assert!(move_timeline_group(&mut m,"A",[false;6],[0.1,0.3],0.3,true,1.).is_err());assert_eq!(m,old);assert_eq!(move_timeline_group(&mut m,"A",[false;6],[0.1,0.3],0.1,true,1.).unwrap(),1);assert_eq!(delete_timeline_events(&mut m,"A",[0.,0.4]),1);}}
