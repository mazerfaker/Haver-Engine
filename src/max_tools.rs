use crate::{Animation,Object,Project};
use serde::{Serialize,Deserialize};
use std::{collections::BTreeMap,path::Path,fs};
use uuid::Uuid;
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct AssetEntry {pub id:Uuid,pub bytes:u64,pub references:Vec<String>,pub present:bool}
#[derive(Clone,Default,Serialize,Deserialize)]
struct DependencyDocument { stamp:[u64;3], assets:Vec<(String,String)>, prefabs:Vec<String> }
#[derive(Default,Serialize,Deserialize)]
pub struct AssetDatabase {pub entries:BTreeMap<String,AssetEntry>,#[serde(default)]pub scan_warnings:Vec<String>,#[serde(default)]pub graph:BTreeMap<String,Vec<String>>,#[serde(default)]pub cache_hits:usize,#[serde(default)]pub cache_misses:usize,#[serde(default)]documents:BTreeMap<String,DependencyDocument>}
impl AssetDatabase {
 /// Return reachable resources and unresolved document links; cycles are visited once.
 pub fn transitive_assets(&self,owner:&str)->(Vec<String>,Vec<String>){
  let mut seen=std::collections::BTreeSet::new();let mut assets=std::collections::BTreeSet::new();let mut unresolved=std::collections::BTreeSet::new();let mut todo=vec![owner.to_string()];
  while let Some(node)=todo.pop(){if !seen.insert(node.clone()){continue;}if seen.len()>32768{unresolved.insert("Traversal budget exceeded".into());break;}if let Some(edges)=self.graph.get(&node){for edge in edges{if !seen.contains(edge){todo.push(edge.clone());}}}else if crate::valid_asset(&node){if self.entries.get(&node).is_none_or(|a|!a.present){unresolved.insert(node.clone());}assets.insert(node);}else{unresolved.insert(node);}}
  (assets.into_iter().collect(),unresolved.into_iter().collect())
 }
 pub fn refresh(root:&Path,paths:&[String],active:&Project)->Result<Self,String>{Self::refresh_with_cache(root,paths,active,true)}
 pub fn refresh_with_cache(root:&Path,paths:&[String],active:&Project,use_cache:bool)->Result<Self,String>{
 let file=root.join(".forge-assets.json");if let Ok(meta)=fs::symlink_metadata(&file){if !meta.is_file()||meta.len()>32*1024*1024{return Err("Unsafe or oversized asset database".into());}}let mut db:Self=if file.exists(){serde_json::from_slice(&fs::read(&file).map_err(|e|e.to_string())?).unwrap_or_default()}else{Self::default()};
 db.scan_warnings.clear();db.graph.clear();db.cache_hits=0;db.cache_misses=0;for entry in db.entries.values_mut(){entry.present=false;entry.references.clear();}
 for path in paths {if !crate::valid_asset(path){continue;}let Ok(meta)=fs::symlink_metadata(root.join(path))else{continue;};if !meta.is_file(){continue;}let bytes=meta.len();let e=db.entries.entry(path.clone()).or_insert_with(||AssetEntry{id:Uuid::new_v4(),bytes,references:vec![],present:true});e.bytes=bytes;e.present=true;}
 fn collect(objects:&[Object],library:&str,stamp:[u64;3])->DependencyDocument{let mut doc=DependencyDocument{stamp,..Default::default()};for o in objects{for p in o.sprite.iter().chain(o.production.normal_map.iter()).chain(o.production.particle_texture.iter()).chain(o.pro.sound.iter().map(|s|&s.path)){if crate::valid_asset(p){doc.assets.push((p.clone(),o.name.clone()));}}if let Some(link)=&o.prefab_link{if crate::valid_prefab_file(&link.file){doc.prefabs.push(format!("{library}{}",link.file));}}}doc.prefabs.sort();doc.prefabs.dedup();doc}
 fn refs(db:&mut AssetDatabase,doc:&DependencyDocument,label:&str){let mut edges=doc.prefabs.clone();for (p,name) in &doc.assets{let e=db.entries.entry(p.clone()).or_insert_with(||AssetEntry{id:Uuid::new_v4(),bytes:0,references:vec![],present:false});e.references.push(format!("{label}: {name}"));edges.push(p.clone());}edges.sort();edges.dedup();db.graph.insert(label.into(),edges);}
 fn walk(db:&mut AssetDatabase,root:&Path,dir:&Path,depth:usize,visited:&mut usize,old:&mut BTreeMap<String,DependencyDocument>)->Result<(),String>{
 if depth>24{return Err("Dependency scan exceeds 24 directory levels".into());}for entry in fs::read_dir(dir).map_err(|e|e.to_string())?{let entry=entry.map_err(|e|e.to_string())?;*visited+=1;if *visited>32768{return Err("Dependency scan exceeds 32768 entries".into());}let p=entry.path();let t=entry.file_type().map_err(|e|e.to_string())?;if t.is_dir(){if entry.file_name()!="target"&&!entry.file_name().to_string_lossy().starts_with('.') {walk(db,root,&p,depth+1,visited,old)?;}continue;}if !t.is_file(){continue;}
 let rel=p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\',"/");let meta=entry.metadata().map_err(|e|e.to_string())?;if crate::valid_asset(&rel)&&p.extension().and_then(|x|x.to_str()).is_some_and(|x|matches!(x.to_ascii_lowercase().as_str(),"png"|"jpg"|"jpeg"|"wav"|"ogg")){let bytes=meta.len();let asset=db.entries.entry(rel.clone()).or_insert_with(||AssetEntry{id:Uuid::new_v4(),bytes,references:vec![],present:true});asset.bytes=bytes;asset.present=true;}
 let scene=rel.ends_with(".forge.json");let prefab=rel.ends_with(".prefab.json");if !scene&&!prefab{continue;}if meta.len()>32*1024*1024{return Err(format!("Dependency source exceeds 32 MiB: {rel}"));}let stamp=meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|t|[meta.len(),t.as_secs(),u64::from(t.subsec_nanos())]);
 let parent=rel.rsplit_once('/').map(|(p,_)|format!("{p}/")).unwrap_or_default();let library=if scene{format!("{parent}prefabs/")}else{parent};
 let cached=old.remove(&rel).filter(|d|Some(d.stamp)==stamp);let doc=if let Some(d)=cached{db.cache_hits+=1;Some(d)}else{db.cache_misses+=1;let tag=stamp.unwrap_or([0;3]);if scene{Project::load(&p).ok().map(|project|collect(&project.objects,&library,tag))}else if let Ok(v)=serde_json::from_slice::<Vec<Object>>(&fs::read(&p).map_err(|e|e.to_string())?){Some(collect(&v,&library,tag))}else if let(Some(parent),Some(name))=(p.parent(),p.file_name().and_then(|x|x.to_str())){crate::read_composite_for_edit(parent,name).ok().map(|doc|{let mut result=collect(&doc.objects,&library,tag);for n in doc.nested{if crate::valid_prefab_file(&n.file){result.prefabs.push(format!("{library}{}",n.file));}}result.prefabs.sort();result.prefabs.dedup();result})}else{None}};
 if let Some(doc)=doc{refs(db,&doc,&rel);if stamp.is_some()&&db.documents.len()<4096{db.documents.insert(rel,doc);}}else if db.scan_warnings.len()<256{db.scan_warnings.push(format!("{}: {rel}",if scene{"Could not parse scene"}else{"Could not parse prefab"}));}
 }Ok(())}
 let mut old=std::mem::take(&mut db.documents);if !use_cache{old.clear();}let mut visited=0;walk(&mut db,root,root,0,&mut visited,&mut old)?;refs(&mut db,&collect(&active.objects,"prefabs/",[0;3]),"Active scene (memory)");
 if let Ok(base)=root.join("assets").canonicalize(){for(rel,entry)in db.entries.iter_mut(){if crate::valid_asset(rel){if let Ok(path)=root.join(rel).canonicalize(){if path.starts_with(&base){if let Ok(meta)=fs::metadata(path){if meta.is_file(){entry.present=true;entry.bytes=meta.len();}}}}}}}
 db.entries.retain(|_,entry|entry.present||!entry.references.is_empty());
 // Cache is disposable. Drop parsed-document cache rather than exceeding file budget.
 let mut data=serde_json::to_vec(&db).map_err(|e|e.to_string())?;if data.len()>32*1024*1024{db.documents.clear();data=serde_json::to_vec(&db).map_err(|e|e.to_string())?;}if data.len()>32*1024*1024{return Err("Asset database exceeds 32 MiB".into());}if !fs::read(&file).is_ok_and(|old|old==data){fs::write(file,data).map_err(|e|e.to_string())?;}Ok(db)
 }
}
#[cfg(test)]mod dependency32{use super::*;#[test]fn cycle_and_missing(){let mut db=AssetDatabase::default();db.graph.insert("a".into(),vec!["b".into(),"assets/a.png".into()]);db.graph.insert("b".into(),vec!["a".into(),"missing".into(),"assets/b.wav".into()]);let(a,m)=db.transitive_assets("a");assert_eq!(a,vec!["assets/a.png","assets/b.wav"]);assert_eq!(m,vec!["missing"]);}}
#[derive(Deserialize)] struct Atlas{frames:Vec<Frame>,meta:Meta}
#[derive(Deserialize)] struct Frame{frame:Bounds,duration:u32,#[serde(default)]rotated:bool,#[serde(default)]trimmed:bool}
#[derive(Deserialize)] struct Bounds{x:u32,y:u32,w:u32,h:u32}
#[derive(Deserialize)] struct Meta{image:String,#[serde(rename="frameTags",default)]tags:Vec<Tag>}
#[derive(Deserialize)] struct Tag{name:String,from:usize,to:usize,direction:String}
/// Normalize Aseprite JSON Array export to a single-row atlas, retaining tag order and per-frame timing.
pub fn import_aseprite(json:&Path,root:&Path)->Result<Object,String>{
 let a:Atlas=serde_json::from_slice(&fs::read(json).map_err(|e|e.to_string())?).map_err(|e|format!("Use Aseprite JSON Array: {e}"))?;
 if a.frames.is_empty()||a.frames.len()>512{return Err("Expected 1–512 frames".into());}let first=&a.frames[0].frame;let(w,h)=(first.w,first.h);if w==0||h==0||a.frames.iter().any(|f|f.rotated||f.trimmed||f.frame.w!=w||f.frame.h!=h){return Err("Export equal-sized, untrimmed, unrotated frames".into());}
 let image_path=Path::new(&a.meta.image);if image_path.components().count()!=1||!matches!(image_path.components().next(),Some(std::path::Component::Normal(_))){return Err("Atlas image must be beside JSON".into());}let source=image::open(json.parent().ok_or("Missing JSON folder")?.join(image_path)).map_err(|e|e.to_string())?.to_rgba8();
 let mut clips=BTreeMap::new();let mut order=Vec::new();let tags=if a.meta.tags.is_empty(){vec![Tag{name:"Idle".into(),from:0,to:a.frames.len()-1,direction:"forward".into()}]}else{a.meta.tags};
 for tag in tags{if tag.from>tag.to||tag.to>=a.frames.len()||clips.contains_key(&tag.name){return Err("Invalid or duplicate tag".into());}let mut indices:Vec<_>=(tag.from..=tag.to).collect();match tag.direction.as_str(){"forward"=>{},"reverse"=>indices.reverse(),"pingpong"=>{let back=indices.iter().rev().skip(1).take(indices.len().saturating_sub(2)).copied().collect::<Vec<_>>();indices.extend(back);},"pingpong_reverse"=>{indices.reverse();let back=indices.iter().rev().skip(1).take(indices.len().saturating_sub(2)).copied().collect::<Vec<_>>();indices.extend(back);},_=>return Err("Unsupported tag direction".into())};let start=order.len();for i in indices{order.push(i);if order.len()>2048{return Err("Too many expanded frames".into());}}let durations=order[start..].iter().map(|&i|a.frames[i].duration.max(1)).collect();clips.insert(tag.name,Animation{columns:1,rows:1,start:start as u32,frames:(order.len()-start) as u32,fps:10.,looping:true,durations,regions:vec![],frame_geometry:Default::default()});}
 let columns=(8192/w).min(order.len() as u32).max(1);let rows=(order.len() as u32).div_ceil(columns);if w>8192||h.checked_mul(rows).unwrap_or(u32::MAX)>8192||u64::from(w)*u64::from(h)*u64::from(columns)*u64::from(rows)>32_000_000{return Err("Expanded atlas too large".into());}let mut dst=image::RgbaImage::new(w*columns,h*rows);for (j,&i) in order.iter().enumerate(){let b=&a.frames[i].frame;if b.x.checked_add(w).is_none_or(|x|x>source.width())||b.y.checked_add(h).is_none_or(|y|y>source.height()){return Err("Frame outside atlas".into());}let frame=image::imageops::crop_imm(&source,b.x,b.y,w,h).to_image();image::imageops::replace(&mut dst,&frame,i64::from(j as u32%columns*w),i64::from(j as u32/columns*h));}for c in clips.values_mut(){c.columns=columns;c.rows=rows;}
 let rel=format!("assets/{}.png",Uuid::new_v4());fs::create_dir_all(root.join("assets")).map_err(|e|e.to_string())?;dst.save(root.join(&rel)).map_err(|e|e.to_string())?;let mut o=Object::new(json.file_stem().and_then(|s|s.to_str()).unwrap_or("Aseprite"),[0.,0.],[w as f32,h as f32],[255;4]);o.sprite=Some(rel);let (name,clip)=clips.first_key_value().ok_or("No tags")?;o.clip_name=name.clone();o.animation=clip.clone();o.clips=clips;Ok(o)
}
#[cfg(test)]mod tests{use super::*;#[test]fn timed_frames(){let a=Animation{columns:2,frames:2,durations:vec![100,300],..Default::default()};assert_eq!(a.frame(0.05),0);assert_eq!(a.frame(0.2),1);assert_eq!(a.frame(0.41),0);}}

#[cfg(test)]mod missing33{use super::*;#[test]fn missing_asset_report(){let mut db=AssetDatabase::default();db.graph.insert("scene".into(),vec!["assets/missing.png".into()]);db.entries.insert("assets/missing.png".into(),AssetEntry{id:Uuid::new_v4(),bytes:0,references:vec!["scene".into()],present:false});let(a,m)=db.transitive_assets("scene");assert_eq!(a,m);}}

#[derive(Clone,Debug)]pub struct AssetGraphNode{pub path:String,pub parent:Option<usize>,pub depth:usize,pub missing:bool,pub repeated:bool}
impl AssetDatabase{
 pub fn dependency_nodes(&self,owner:&str)->Vec<AssetGraphNode>{
  let mut rows=Vec::new();let mut seen=std::collections::BTreeSet::new();let mut queue=std::collections::VecDeque::from([(owner.to_owned(),None,0usize)]);
  while let Some((path,parent,depth))=queue.pop_front(){if rows.len()>=512{break;}let repeated=!seen.insert(path.clone());let edges=self.graph.get(&path);let missing=edges.is_none()&&self.entries.get(&path).is_none_or(|a|!a.present);let i=rows.len();rows.push(AssetGraphNode{path,parent,depth,missing,repeated});if !repeated&&depth<16{if let Some(edges)=edges{let room=512usize.saturating_sub(rows.len()+queue.len());queue.extend(edges.iter().take(room).map(|s|(s.clone(),Some(i),depth+1)));}}}
  rows
 }
}
#[cfg(test)]mod graph35{use super::*;#[test]fn nested_and_missing(){let mut db=AssetDatabase::default();db.graph.insert("root".into(),vec!["assets/child.prefab.json".into()]);db.graph.insert("assets/child.prefab.json".into(),vec!["root".into(),"assets/missing.png".into()]);let(a,m)=db.transitive_assets("root");assert_eq!(a,vec!["assets/missing.png"]);assert_eq!(m,a);let rows=db.dependency_nodes("root");assert_eq!(rows.len(),4);assert!(rows.iter().any(|r|r.repeated));assert!(rows.iter().any(|r|r.missing));}}
