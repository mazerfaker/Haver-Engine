use crate::Object;
use std::collections::{HashMap,HashSet};
pub fn validate_prefab(objects:&[Object])->Result<(),String>{
 if objects.is_empty()||objects.len()>10000{return Err("Prefab must contain 1–10000 objects".into());}
 let map:HashMap<_,_>=objects.iter().map(|o|(o.id,o.parent)).collect();if map.len()!=objects.len(){return Err("Duplicate prefab ID".into());}
 for o in objects{if !o.motion.valid_clip_keys(){return Err("Invalid clip transform channels".into());}if !o.position.iter().chain(o.size.iter()).chain([&o.rotation]).all(|v|v.is_finite())||o.size.iter().any(|v|*v<=0.){return Err("Invalid prefab transform".into());}
 for path in [&o.sprite,&o.production.normal_map,&o.production.particle_texture].into_iter().flatten(){if !crate::valid_asset(path){return Err("Invalid prefab asset path".into());}}
 if let Some(sound)=&o.pro.sound{if !sound.path.is_empty()&&!crate::valid_asset(&sound.path){return Err("Invalid prefab sound path".into());}}
 let mut seen=HashSet::from([o.id]);let mut parent=o.parent;while let Some(id)=parent{if !seen.insert(id){return Err("Prefab hierarchy cycle".into());}parent=*map.get(&id).ok_or("Missing prefab parent")?;}}
 Ok(())
}
pub fn instantiate_checked(objects:Vec<Object>,offset:[f32;2])->Result<Vec<Object>,String>{validate_prefab(&objects)?;if !offset.iter().all(|v|v.is_finite()){return Err("Invalid prefab offset".into());}let mut result=crate::instantiate(objects);for o in &mut result{for i in 0..2{o.position[i]+=offset[i];for k in o.motion.keys.iter_mut().chain(o.motion.clip_keys.values_mut().flatten()){k.position[i]+=offset[i];}}}validate_prefab(&result)?;Ok(result)}
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
 let local_sources:HashMap<_,_>=local.iter().map(|o|(o.id,o.prefab_link.as_ref().unwrap().source)).collect();
 let file=&local[0].prefab_link.as_ref().unwrap().file;
 for old in &local {let link=old.prefab_link.as_ref().unwrap();if &link.file!=file{return Err("Mixed prefab sources in instance".into());}let parent=old.parent.and_then(|id|local_sources.get(&id).copied());if parent!=source_map[&link.source].parent{return Err("Prefab hierarchy changed; refresh refused".into());}}
 let mut replacements=Vec::new();for old in local{let link=old.prefab_link.as_ref().unwrap();let mut next=(*source_map[&link.source]).clone();next.id=old.id;next.parent=old.parent;next.position=old.position;next.rotation=old.rotation;next.size=old.size;next.prefab_link=Some(link.clone());let keep=|name:&str|link.overrides.iter().any(|s|s==name);
 if keep("appearance"){next.motion.pivot=old.motion.pivot;next.name=old.name.clone();next.sprite=old.sprite.clone();next.color=old.color;next.layer=old.layer;next.production.normal_map=old.production.normal_map.clone();}
 if keep("animation"){next.animation=old.animation.clone();next.clips=old.clips.clone();next.clip_name=old.clip_name.clone();next.motion=old.motion.clone();}else{next.motion.keys=old.motion.keys.clone();next.motion.clip_keys=old.motion.clip_keys.clone();}
 if keep("physics"){next.body=old.body;next.material=old.material.clone();next.pro.physics=old.pro.physics.clone();next.tile=old.tile.clone();}
 if keep("gameplay"){next.pro.pickup=old.pro.pickup.clone();next.pro.combat=old.pro.combat.clone();next.pro.light=old.pro.light.clone();next.pro.emitter=old.pro.emitter.clone();next.pro.sound=old.pro.sound.clone();next.pro.variables=old.pro.variables.clone();next.production.particle_texture=old.production.particle_texture.clone();next.production.navigation=old.production.navigation.clone();next.player=old.player;next.camera=old.camera;}
 if keep("scripts"){next.script=old.script.clone();next.graph=old.graph.clone();next.pro.graph_positions=old.pro.graph_positions.clone();}
 replacements.push(next);}
 let n=replacements.len();let by_id:HashMap<_,_>=replacements.into_iter().map(|o|(o.id,o)).collect();for o in &mut project.objects{if let Some(next)=by_id.get(&o.id){*o=next.clone();}}Ok(n)
}
#[cfg(test)]mod linked_tests{use super::*;#[test]fn refresh_preserves_local(){let mut source=Object::new("source",[0.;2],[16.;2],[255;4]);let mut p=crate::Project::default();let mut o=instantiate_linked(vec![source.clone()],"a.prefab.json",[8.,9.]).unwrap().remove(0);let id=o.id;let link=o.prefab_link.as_mut().unwrap();let instance=link.instance;link.overrides.push("appearance".into());o.name="local".into();p.objects.push(o);source.name="updated".into();source.script="new".into();assert_eq!(refresh_linked(&mut p,instance,&[source]).unwrap(),1);let o=p.objects.last().unwrap();assert_eq!(o.id,id);assert_eq!(o.name,"local");assert_eq!(o.position,[8.,9.]);assert_eq!(o.script,"new");}}

#[cfg(test)]mod refresh_regressions{use super::*;
 #[test]fn changed_parent_is_atomic(){let a=Object::new("a",[0.;2],[8.;2],[255;4]);let mut b=Object::new("b",[0.;2],[8.;2],[255;4]);b.parent=Some(a.id);let mut p=crate::Project::default();let linked=instantiate_linked(vec![a.clone(),b.clone()],"a.prefab.json",[0.;2]).unwrap();let id=linked[0].prefab_link.as_ref().unwrap().instance;p.objects.extend(linked);let before=p.clone();b.parent=None;assert!(refresh_linked(&mut p,id,&[a,b]).is_err());assert_eq!(p,before);}
 #[test]fn gameplay_override_does_not_override_physics(){let mut a=Object::new("a",[0.;2],[8.;2],[255;4]);let mut p=crate::Project::default();let mut linked=instantiate_linked(vec![a.clone()],"a.prefab.json",[0.;2]).unwrap().remove(0);let link=linked.prefab_link.as_mut().unwrap();let id=link.instance;link.overrides.push("gameplay".into());linked.pro.variables.insert("local".into(),5.);p.objects.push(linked);a.body=crate::Body::Static;refresh_linked(&mut p,id,&[a]).unwrap();let o=p.objects.last().unwrap();assert_eq!(o.body,crate::Body::Static);assert_eq!(o.pro.variables.get("local"),Some(&5.));}
}

/// Opt-in topology growth. Preserve existing children and overrides, reject deletions and reparenting.
pub fn refresh_linked_additions(project:&mut crate::Project,instance:uuid::Uuid,source:&[Object])->Result<usize,String>{
 validate_prefab(source)?;let mut next=project.clone();let local:Vec<_>=next.objects.iter().filter(|o|o.prefab_link.as_ref().is_some_and(|l|l.instance==instance)).cloned().collect();let first=local.first().ok_or("No linked instance")?;let file=first.prefab_link.as_ref().unwrap().file.clone();let sources:HashMap<_,_>=source.iter().map(|o|(o.id,o)).collect();let mut ids:HashMap<_,_>=local.iter().map(|o|(o.prefab_link.as_ref().unwrap().source,o.id)).collect();if ids.len()!=local.len()||local.iter().any(|o|o.prefab_link.as_ref().unwrap().file!=file)||ids.keys().any(|id|!sources.contains_key(id)){return Err("Existing prefab members removed or ambiguous".into());}
 let additions=source.len()-local.len();if next.objects.len()+additions>10000{return Err("Scene object limit".into());}
 for o in &local{let s=sources[&o.prefab_link.as_ref().unwrap().source];if s.parent.and_then(|id|ids.get(&id).copied())!=o.parent && s.parent.is_some(){return Err("Existing prefab parent changed".into());}}
 let mut pending:Vec<_>=source.iter().filter(|o|!ids.contains_key(&o.id)).collect();while !pending.is_empty(){let n=pending.len();pending.retain(|s|{let parent=s.parent.and_then(|id|ids.get(&id).copied());if s.parent.is_some()&&parent.is_none(){return true;}
 let (anchor_source,anchor_local)=if let (Some(sp),Some(lp))=(s.parent,parent){(sources[&sp],next.objects.iter().find(|o|o.id==lp).unwrap())}else{(sources[&first.prefab_link.as_ref().unwrap().source],first)};let delta=[anchor_local.position[0]-anchor_source.position[0],anchor_local.position[1]-anchor_source.position[1]];let mut o=(**s).clone();o.id=uuid::Uuid::new_v4();o.parent=parent;for i in 0..2{o.position[i]+=delta[i];for k in o.motion.keys.iter_mut().chain(o.motion.clip_keys.values_mut().flatten()){k.position[i]+=delta[i];}}o.prefab_link=Some(PrefabLink{file:file.clone(),instance,source:s.id,overrides:vec![]});ids.insert(s.id,o.id);next.objects.push(o);false});if pending.len()==n{return Err("Cannot resolve new prefab hierarchy".into());}}
 let refreshed=refresh_linked(&mut next,instance,source)?;*project=next;Ok(refreshed)
}
#[cfg(test)]mod addition_tests{use super::*;#[test]fn new_nested_children(){let a=Object::new("a",[0.;2],[8.;2],[255;4]);let mut b=Object::new("b",[3.,5.],[8.;2],[255;4]);b.parent=Some(a.id);let mut c=Object::new("c",[6.,7.],[8.;2],[255;4]);c.parent=Some(b.id);let mut p=crate::Project::default();let linked=instantiate_linked(vec![a.clone()],"a.prefab.json",[10.,20.]).unwrap();let id=linked[0].id;let instance=linked[0].prefab_link.as_ref().unwrap().instance;p.objects.extend(linked);assert_eq!(refresh_linked_additions(&mut p,instance,&[a,b,c]).unwrap(),3);let b=p.objects.iter().find(|o|o.name=="b").unwrap();assert_eq!(b.parent,Some(id));assert_eq!(b.position,[13.,25.]);let c=p.objects.iter().find(|o|o.name=="c").unwrap();assert_eq!(c.parent,Some(b.id));assert_eq!(c.position,[16.,27.]);}}

/// Refresh every materialized instance of a single library prefab, atomically across the scene.
pub fn refresh_all_linked(project:&mut crate::Project,file:&str,source:&[Object])->Result<usize,String>{if !valid_prefab_file(file){return Err("Invalid library filename".into());}let ids:std::collections::BTreeSet<_>=project.objects.iter().filter_map(|o|o.prefab_link.as_ref().filter(|l|l.file==file).map(|l|l.instance)).collect();if ids.is_empty(){return Err("No linked instances".into());}let mut candidate=project.clone();let mut n=0;for id in ids{n+=refresh_linked_additions(&mut candidate,id,source)?;}*project=candidate;Ok(n)}
#[cfg(test)]mod bulk_tests{use super::*;#[test]fn refresh_two_instances(){let a=Object::new("a",[0.;2],[8.;2],[255;4]);let mut b=Object::new("b",[1.,2.],[8.;2],[255;4]);b.parent=Some(a.id);let mut p=crate::Project::default();p.objects.extend(instantiate_linked(vec![a.clone()],"a.prefab.json",[10.,0.]).unwrap());p.objects.extend(instantiate_linked(vec![a.clone()],"a.prefab.json",[20.,0.]).unwrap());assert_eq!(refresh_all_linked(&mut p,"a.prefab.json",&[a,b]).unwrap(),4);assert_eq!(p.objects.iter().filter(|o|o.name=="b").count(),2);}}

/// External prefab composition. Legacy array files are still supported.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum LibraryDocument { Legacy(Vec<Object>), Composite{ #[serde(default)] objects:Vec<Object>, #[serde(default)] nested:Vec<NestedPrefab> } }
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct NestedPrefab { pub id:uuid::Uuid,pub file:String,#[serde(default)] pub parent:Option<uuid::Uuid>,#[serde(default)] pub offset:[f32;2] }
/// Stable source identities scoped to a nested reference. Collisions are rejected by validation.
fn nested_id(scope:uuid::Uuid,child:uuid::Uuid)->uuid::Uuid{
 let mut a=0xcbf29ce484222325u64;let mut b=0x84222325cbf29ce4u64;
 for &x in scope.as_bytes().iter().chain(child.as_bytes()){a=(a^u64::from(x)).wrapping_mul(0x100000001b3);b=(b^u64::from(x)).wrapping_mul(0x9e3779b185ebca87);}
 uuid::Uuid::from_u128((u128::from(a)<<64)|u128::from(b))
}
/// Resolve files only within the prefab library; reject cycles, oversized files and deep graphs.
pub fn load_library_prefab(root:&std::path::Path,file:&str)->Result<Vec<Object>,String>{load_library_prefab_with_override(root,file,None)}
fn load_library_prefab_with_override(root:&std::path::Path,file:&str,replace:Option<(&std::path::Path,&[u8])>)->Result<Vec<Object>,String>{
 let root=root.canonicalize().map_err(|e|e.to_string())?;let mut stack=HashSet::new();let mut reads=0usize;
 fn resolve(root:&std::path::Path,file:&str,stack:&mut HashSet<std::path::PathBuf>,reads:&mut usize,depth:usize,replace:Option<(&std::path::Path,&[u8])>)->Result<Vec<Object>,String>{
  use std::io::Read;
  if depth>8||*reads>=128||!valid_prefab_file(file){return Err("Invalid nested prefab path or expansion limit".into());}*reads+=1;
  let path=root.join(file).canonicalize().map_err(|e|e.to_string())?;if !path.starts_with(root)||!stack.insert(path.clone()){return Err("Nested prefab cycle or path outside library".into());}
  let mut data=Vec::new();if let Some((_,bytes))=replace.filter(|(target,_)|*target==path.as_path()){data.extend_from_slice(bytes);}else{std::fs::File::open(&path).map_err(|e|e.to_string())?.take(8*1024*1024+1).read_to_end(&mut data).map_err(|e|e.to_string())?;}if data.len()>8*1024*1024{return Err("Prefab file exceeds 8 MiB".into());}
  let (mut objects,nested)=match serde_json::from_slice::<LibraryDocument>(&data).map_err(|e|e.to_string())?{LibraryDocument::Legacy(v)=>(v,vec![]),LibraryDocument::Composite{objects,nested}=>(objects,nested)};
  let mut refs=HashSet::new();let local_ids:HashSet<_>=objects.iter().map(|o|o.id).collect();
  for n in nested {if !refs.insert(n.id)||!n.offset.iter().all(|v|v.is_finite())||n.parent.is_some_and(|p|!local_ids.contains(&p)){return Err("Invalid nested reference identity, offset or parent".into());}
   let mut children=resolve(root,&n.file,stack,reads,depth+1,replace)?;if objects.len().saturating_add(children.len())>10000{return Err("Prefab object limit".into());}
   let ids:HashMap<_,_>=children.iter().map(|o|(o.id,nested_id(n.id,o.id))).collect();
   for o in &mut children {o.id=ids[&o.id];o.parent=o.parent.map(|p|ids[&p]).or(n.parent);o.prefab_link=None;for i in 0..2{o.position[i]+=n.offset[i];for k in o.motion.keys.iter_mut().chain(o.motion.clip_keys.values_mut().flatten()){k.position[i]+=n.offset[i];}}}
   objects.extend(children);
  }
  stack.remove(&path);validate_prefab(&objects)?;Ok(objects)
 }
 resolve(&root,file,&mut stack,&mut reads,0,replace)
}
#[cfg(test)]mod external_tests{use super::*;#[test]fn nested_load_stable_and_cycle_safe(){let root=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());std::fs::create_dir_all(&root).unwrap();let child=Object::new("child",[1.,2.],[8.;2],[255;4]);std::fs::write(root.join("child.prefab.json"),serde_json::to_vec(&vec![child]).unwrap()).unwrap();let parent=Object::new("parent",[0.;2],[8.;2],[255;4]);let doc=serde_json::json!({"objects":[parent.clone()],"nested":[{"id":uuid::Uuid::new_v4(),"file":"child.prefab.json","parent":parent.id,"offset":[4.,5.]}]});std::fs::write(root.join("a.prefab.json"),serde_json::to_vec(&doc).unwrap()).unwrap();let a=load_library_prefab(&root,"a.prefab.json").unwrap();let b=load_library_prefab(&root,"a.prefab.json").unwrap();assert_eq!(a,b);assert_eq!(a[1].parent,Some(parent.id));assert_eq!(a[1].position,[5.,7.]);let cycle=serde_json::json!({"nested":[{"id":uuid::Uuid::new_v4(),"file":"cycle.prefab.json"}]});std::fs::write(root.join("cycle.prefab.json"),serde_json::to_vec(&cycle).unwrap()).unwrap();assert!(load_library_prefab(&root,"cycle.prefab.json").is_err());std::fs::remove_dir_all(root).unwrap();}}

/// Create a new composite document without overwriting any source prefab. Resolve it before accepting.
pub fn create_composite_prefab(root:&std::path::Path,file:&str,objects:&[Object],nested:&[NestedPrefab])->Result<usize,String>{
 use std::io::Write;
 if !valid_prefab_file(file)||nested.is_empty()||nested.len()>128{return Err("Invalid composite filename or reference count".into());}
 if !objects.is_empty(){validate_prefab(objects)?;}
 if nested.iter().any(|n|!valid_prefab_file(&n.file)||n.file==file){return Err("Invalid or recursive reference".into());}
 std::fs::create_dir_all(root).map_err(|e|e.to_string())?;
 let root=root.canonicalize().map_err(|e|e.to_string())?;let path=root.join(file);
 let bytes=serde_json::to_vec_pretty(&serde_json::json!({"objects":objects,"nested":nested})).map_err(|e|e.to_string())?;
 if bytes.len()>8*1024*1024{return Err("Composite exceeds 8 MiB".into());}
 let mut handle=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e|e.to_string())?;
 let written=handle.write_all(&bytes).and_then(|_|handle.sync_all());drop(handle);
 let result=written.map_err(|e|e.to_string()).and_then(|_|load_library_prefab(&root,file).map(|v|v.len()));
 if result.is_err(){let _=std::fs::remove_file(&path);}result
}
#[cfg(test)]mod composer_tests{use super::*;#[test]fn preserves_source_and_no_overwrite(){let root=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());std::fs::create_dir_all(&root).unwrap();let source=serde_json::to_vec(&vec![Object::new("child",[0.;2],[1.;2],[255;4])]).unwrap();std::fs::write(root.join("child.prefab.json"),&source).unwrap();let n=NestedPrefab{id:uuid::Uuid::new_v4(),file:"child.prefab.json".into(),parent:None,offset:[4.,5.]};assert_eq!(create_composite_prefab(&root,"parent.prefab.json",&[],&[n.clone()]).unwrap(),1);assert!(create_composite_prefab(&root,"child.prefab.json",&[],&[n.clone()]).is_err());assert!(create_composite_prefab(&root,"parent.prefab.json",&[],&[n]).is_err());assert_eq!(std::fs::read(root.join("child.prefab.json")).unwrap(),source);std::fs::remove_dir_all(root).unwrap();}}

/// An editable composite with the original bytes for optimistic conflict detection.
#[derive(Clone,Debug)]
pub struct CompositeEdit{pub objects:Vec<Object>,pub nested:Vec<NestedPrefab>,original:Vec<u8>}
pub fn read_composite_for_edit(root:&std::path::Path,file:&str)->Result<CompositeEdit,String>{
 use std::io::Read;
 if !valid_prefab_file(file){return Err("Invalid prefab filename".into());}let root=root.canonicalize().map_err(|e|e.to_string())?;let path=root.join(file).canonicalize().map_err(|e|e.to_string())?;if path!=root.join(file){return Err("Editing redirected prefab paths is disabled".into());}
 let mut data=Vec::new();std::fs::File::open(path).map_err(|e|e.to_string())?.take(8*1024*1024+1).read_to_end(&mut data).map_err(|e|e.to_string())?;if data.len()>8*1024*1024{return Err("Prefab exceeds 8 MiB".into());}
 match serde_json::from_slice::<LibraryDocument>(&data).map_err(|e|e.to_string())?{LibraryDocument::Legacy(_)=>Err("Open a composite prefab, not a legacy object list".into()),LibraryDocument::Composite{objects,nested}=>Ok(CompositeEdit{objects,nested,original:data})}
}
pub fn update_composite_prefab(root:&std::path::Path,file:&str,doc:&CompositeEdit,nested:&[NestedPrefab])->Result<usize,String>{
 use std::io::Write;
 if nested.len()>128{return Err("Nested reference limit".into());}let current=read_composite_for_edit(root,file)?;if current.original!=doc.original{return Err("Prefab changed externally; reopen before saving".into());}
 let root=root.canonicalize().map_err(|e|e.to_string())?;let path=root.join(file);let bytes=serde_json::to_vec_pretty(&serde_json::json!({"objects":doc.objects,"nested":nested})).map_err(|e|e.to_string())?;if bytes.len()>8*1024*1024{return Err("Composite exceeds 8 MiB".into());}
 let count=load_library_prefab_with_override(&root,file,Some((&path,&bytes)))?.len();
 let token=uuid::Uuid::new_v4();let backup=root.join(format!("{file}.{token}.bak"));let temporary=root.join(format!("{file}.{token}.tmp"));
 let write=|path:&std::path::Path,data:&[u8]|->Result<(),String>{let mut f=std::fs::OpenOptions::new().create_new(true).write(true).open(path).map_err(|e|e.to_string())?;f.write_all(data).and_then(|_|f.sync_all()).map_err(|e|e.to_string())};
 write(&backup,&current.original)?;if let Err(e)=write(&temporary,&bytes){let _=std::fs::remove_file(&temporary);return Err(e);}if read_composite_for_edit(&root,file).map(|v|v.original).ok().as_ref()!=Some(&current.original){let _=std::fs::remove_file(&temporary);return Err("Concurrent prefab edit; backup retained".into());}
 if let Err(e)=std::fs::rename(&temporary,&path){let _=std::fs::remove_file(&temporary);return Err(format!("Replace failed: {e}; original and backup retained"));}Ok(count)
}
#[cfg(test)]mod edit_composite_tests{use super::*;#[test]fn edit_and_reject_cycle(){let root=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());std::fs::create_dir_all(&root).unwrap();std::fs::write(root.join("child.prefab.json"),serde_json::to_vec(&vec![Object::new("child",[0.;2],[1.;2],[255;4])]).unwrap()).unwrap();let mut n=NestedPrefab{id:uuid::Uuid::new_v4(),file:"child.prefab.json".into(),parent:None,offset:[0.;2]};create_composite_prefab(&root,"a.prefab.json",&[],&[n.clone()]).unwrap();let doc=read_composite_for_edit(&root,"a.prefab.json").unwrap();let before=load_library_prefab(&root,"a.prefab.json").unwrap();n.offset=[8.,4.];update_composite_prefab(&root,"a.prefab.json",&doc,&[n.clone()]).unwrap();let after=load_library_prefab(&root,"a.prefab.json").unwrap();assert_eq!(before[0].id,after[0].id);assert_eq!(after[0].position,[8.,4.]);assert!(update_composite_prefab(&root,"a.prefab.json",&doc,&[n.clone()]).is_err());let doc=read_composite_for_edit(&root,"a.prefab.json").unwrap();n.file="a.prefab.json".into();assert!(update_composite_prefab(&root,"a.prefab.json",&doc,&[n]).is_err());assert_eq!(load_library_prefab(&root,"a.prefab.json").unwrap(),after);std::fs::remove_dir_all(root).unwrap();}}
