use crate::Object;
use std::collections::{HashMap,HashSet};
pub fn validate_prefab(objects:&[Object])->Result<(),String>{
 if objects.is_empty()||objects.len()>10000{return Err("Prefab must contain 1–10000 objects".into());}
 let map:HashMap<_,_>=objects.iter().map(|o|(o.id,o.parent)).collect();if map.len()!=objects.len(){return Err("Duplicate prefab ID".into());}
 for o in objects{if !o.position.iter().chain(o.size.iter()).chain([&o.rotation]).all(|v|v.is_finite())||o.size.iter().any(|v|*v<=0.){return Err("Invalid prefab transform".into());}
 for path in [&o.sprite,&o.production.normal_map,&o.production.particle_texture].into_iter().flatten(){if !crate::valid_asset(path){return Err("Invalid prefab asset path".into());}}
 let mut seen=HashSet::from([o.id]);let mut parent=o.parent;while let Some(id)=parent{if !seen.insert(id){return Err("Prefab hierarchy cycle".into());}parent=*map.get(&id).ok_or("Missing prefab parent")?;}}
 Ok(())
}
pub fn instantiate_checked(objects:Vec<Object>,offset:[f32;2])->Result<Vec<Object>,String>{validate_prefab(&objects)?;if !offset.iter().all(|v|v.is_finite()){return Err("Invalid prefab offset".into());}let mut result=crate::instantiate(objects);for o in &mut result{for i in 0..2{o.position[i]+=offset[i];for k in &mut o.motion.keys{k.position[i]+=offset[i];}}}validate_prefab(&result)?;Ok(result)}
#[cfg(test)]mod tests{use super::*;#[test]fn nested_remap(){let a=Object::new("parent",[0.;2],[1.;2],[255;4]);let mut b=Object::new("child",[10.,2.],[1.;2],[255;4]);let mut c=Object::new("nested",[11.,3.],[1.;2],[255;4]);b.parent=Some(a.id);c.parent=Some(b.id);let v=instantiate_checked(vec![a.clone(),b,c],[5.,7.]).unwrap();assert_ne!(a.id,v[0].id);assert_eq!(v[1].parent,Some(v[0].id));assert_eq!(v[2].parent,Some(v[1].id));assert_eq!(v[2].position,[16.,10.]);}#[test]fn rejects_cycle(){let mut a=Object::new("a",[0.;2],[1.;2],[255;4]);a.parent=Some(a.id);assert!(validate_prefab(&[a]).is_err());}}

/// Editor-only link metadata. Scenes retain materialized objects for standalone/export use.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
pub struct PrefabLink{pub file:String,pub instance:uuid::Uuid,pub source:uuid::Uuid,#[serde(default)]pub overrides:Vec<String>}
pub fn valid_prefab_file(file:&str)->bool{file.ends_with(".prefab.json")&&!file.contains('/')&&!file.contains('\\')&&std::path::Path::new(file).components().count()==1&&matches!(std::path::Path::new(file).components().next(),Some(std::path::Component::Normal(_)))}
pub fn instantiate_linked(source:Vec<Object>,file:&str,offset:[f32;2])->Result<Vec<Object>,String>{if !valid_prefab_file(file){return Err("Invalid prefab library filename".into());}let instance=uuid::Uuid::new_v4();let ids:Vec<_>=source.iter().map(|o|o.id).collect();let mut objects=instantiate_checked(source,offset)?;for (o,source) in objects.iter_mut().zip(ids){o.prefab_link=Some(PrefabLink{file:file.into(),instance,source,overrides:vec![]});}Ok(objects)}
/// Sync matching source identities atomically. Keep local identity, hierarchy and transforms;
/// component groups may be explicitly overridden. Refuse topology changes instead of losing local objects.
pub fn refresh_linked(project:&mut crate::Project,instance:uuid::Uuid,source:&[Object])->Result<usize,String>{
 validate_prefab(source)?;let source_map:HashMap<_,_>=source.iter().map(|o|(o.id,o)).collect();let local:Vec<_>=project.objects.iter().filter(|o|o.prefab_link.as_ref().is_some_and(|l|l.instance==instance)).collect();if local.is_empty(){return Err("No linked instance".into());}let keys:HashSet<_>=local.iter().map(|o|o.prefab_link.as_ref().unwrap().source).collect();if keys.len()!=local.len()||keys.len()!=source_map.len()||keys.iter().any(|id|!source_map.contains_key(id)){return Err("Prefab membership changed; insert a new instance or unlink before editing".into());}
 let mut replacements=Vec::new();for old in local{let link=old.prefab_link.as_ref().unwrap();let mut next=(*source_map[&link.source]).clone();next.id=old.id;next.parent=old.parent;next.position=old.position;next.rotation=old.rotation;next.size=old.size;next.prefab_link=Some(link.clone());let keep=|name:&str|link.overrides.iter().any(|s|s==name);
 if keep("appearance"){next.name=old.name.clone();next.sprite=old.sprite.clone();next.color=old.color;next.layer=old.layer;next.production.normal_map=old.production.normal_map.clone();}
 if keep("animation"){next.animation=old.animation.clone();next.clips=old.clips.clone();next.clip_name=old.clip_name.clone();next.motion=old.motion.clone();}else{next.motion.keys=old.motion.keys.clone();}
 if keep("physics"){next.body=old.body;next.material=old.material.clone();next.pro.physics=old.pro.physics.clone();next.tile=old.tile.clone();}
 if keep("gameplay"){next.pro=old.pro.clone();next.production.particle_texture=old.production.particle_texture.clone();next.production.navigation=old.production.navigation.clone();next.player=old.player;next.camera=old.camera;}
 if keep("scripts"){next.script=old.script.clone();next.graph=old.graph.clone();}
 replacements.push(next);}
 let n=replacements.len();let by_id:HashMap<_,_>=replacements.into_iter().map(|o|(o.id,o)).collect();for o in &mut project.objects{if let Some(next)=by_id.get(&o.id){*o=next.clone();}}Ok(n)
}
#[cfg(test)]mod linked_tests{use super::*;#[test]fn refresh_preserves_local(){let mut source=Object::new("source",[0.;2],[16.;2],[255;4]);let mut p=crate::Project::default();let mut o=instantiate_linked(vec![source.clone()],"a.prefab.json",[8.,9.]).unwrap().remove(0);let id=o.id;let link=o.prefab_link.as_mut().unwrap();let instance=link.instance;link.overrides.push("appearance".into());o.name="local".into();p.objects.push(o);source.name="updated".into();source.script="new".into();assert_eq!(refresh_linked(&mut p,instance,&[source]).unwrap(),1);let o=p.objects.last().unwrap();assert_eq!(o.id,id);assert_eq!(o.name,"local");assert_eq!(o.position,[8.,9.]);assert_eq!(o.script,"new");}}
