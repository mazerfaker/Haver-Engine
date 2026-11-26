use crate::{Object,Project};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Motion { pub keys:Vec<Key>,pub pivot:[f32;2],pub hitboxes:std::collections::BTreeMap<u32,[f32;4]>,pub auto_state:bool, pub transitions:Vec<crate::ClipTransition> }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Key {pub time:f32,pub position:[f32;2],pub rotation:f32,pub smooth:bool,#[serde(default)]pub bezier:Option<[[f32;2];3]>}
impl Motion {pub fn sample(&self,t:f32)->Option<([f32;2],f32)>{let mut k=self.keys.iter().filter(|k|k.time.is_finite()).collect::<Vec<_>>();k.sort_by(|a,b|a.time.total_cmp(&b.time));let a=*k.first()?;if t<=a.time{return Some((a.position,a.rotation));}for pair in k.windows(2){let(a,b)=(pair[0],pair[1]);if t<=b.time{let mut u=((t-a.time)/(b.time-a.time).max(0.00001)).clamp(0.,1.);if a.smooth{u=u*u*(3.-2.*u);}let raw=((t-a.time)/(b.time-a.time).max(0.00001)).clamp(0.,1.);let blend=|i:usize|{if let Some(handles)=a.bezier{let [c1,c2]=handles[i];let v=1.-raw;3.*v*v*raw*c1+3.*v*raw*raw*c2+raw*raw*raw}else{u}};return Some((std::array::from_fn(|i|a.position[i]+(b.position[i]-a.position[i])*blend(i)),a.rotation+(b.rotation-a.rotation)*blend(2)));}}let b=k.last()?;Some((b.position,b.rotation))}}
pub fn prefab(p:&Project,id:Uuid)->Vec<Object>{let mut ids=std::collections::HashSet::from([id]);loop{let n=ids.len();for o in &p.objects{if o.parent.is_some_and(|x|ids.contains(&x)){ids.insert(o.id);}}if ids.len()==n{break;}}p.objects.iter().filter(|o|ids.contains(&o.id)).cloned().map(|mut o|{if o.id==id{o.parent=None;}o}).collect()}
pub fn instantiate(mut objects:Vec<Object>)->Vec<Object>{let map:std::collections::HashMap<_,_>=objects.iter().map(|o|(o.id,Uuid::new_v4())).collect();for o in &mut objects{o.id=map[&o.id];o.parent=o.parent.and_then(|p|map.get(&p).copied());}objects}
#[cfg(test)]mod tests{use super::*;#[test]fn interpolation(){let m=Motion{keys:vec![Key{time:0.,position:[0.,0.],rotation:0.,smooth:false,bezier:None},Key{time:2.,position:[10.,20.],rotation:90.,smooth:false,bezier:None}],..Default::default()};assert_eq!(m.sample(1.),Some(([5.,10.],45.)));}#[test]fn prefab_ids(){let p=Project::default();let a=instantiate(prefab(&p,p.objects[0].id));assert_ne!(a[0].id,p.objects[0].id);}}
