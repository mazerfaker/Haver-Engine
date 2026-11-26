//! Versioned checkpoint, separate from authored scenes. Physics/Lua VM and GPU state are rebuilt.
use crate::{Project,Inventory,DemoSession};
use serde::{Serialize,Deserialize};
use std::{path::Path,io::{Read,Write}};
const MAX_BYTES:u64=32*1024*1024;
/// Slot 1 intentionally uses the original checkpoint filename for backward compatibility.
pub const SLOT_COUNT:usize=3;
pub fn slot_path(entry:&Path,slot:usize)->Result<std::path::PathBuf,String>{
 match slot {
  1=>Ok(entry.with_extension("checkpoint.json")),
  2..=SLOT_COUNT=>Ok(entry.with_extension(format!("checkpoint.slot{slot}.json"))),
  _=>Err("Invalid checkpoint slot".into()),
 }
}
#[derive(Serialize,Deserialize)]
pub struct Checkpoint {pub version:u32,pub entry:String,pub level:usize,pub project:Project,pub inventory:Option<Inventory>,pub demo:Option<DemoSession>,pub elapsed:f32,pub finished:bool}
impl Checkpoint {
 pub fn validate(&self,entry:&str)->Result<(),String>{if self.version!=1||self.entry!=entry||!(1..=4).contains(&self.level)||!self.elapsed.is_finite()||self.elapsed<0.||self.project.objects.len()>100000{return Err("Invalid or incompatible checkpoint".into());}self.project.validate_data()?;crate::modules::validate(&self.project)?;if self.inventory.as_ref().is_some_and(|i|!i.equipment_valid(&self.project.settings.item_catalog)){return Err("Invalid checkpoint inventory".into());}if self.demo.as_ref().is_some_and(|d|d.health>3||!d.cooldown.is_finite()||!d.attack_cooldown.is_finite()||!d.facing.is_finite()){return Err("Invalid checkpoint demo".into());}Ok(())}
 pub fn save(&self,path:&Path)->Result<(),String>{self.validate(&self.entry)?;let bytes=serde_json::to_vec(self).map_err(|e|e.to_string())?;if bytes.len() as u64>MAX_BYTES{return Err("Checkpoint exceeds 32 MiB".into());}let tmp=path.with_extension(format!("{}.tmp",uuid::Uuid::new_v4()));let result=(||->Result<(),String>{let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp).map_err(|e|e.to_string())?;f.write_all(&bytes).map_err(|e|e.to_string())?;f.sync_all().map_err(|e|e.to_string())?;drop(f);std::fs::rename(&tmp,path).map_err(|e|e.to_string())})();if result.is_err(){let _=std::fs::remove_file(tmp);}result}
 pub fn load(path:&Path,entry:&str)->Result<Self,String>{let f=std::fs::File::open(path).map_err(|e|e.to_string())?;let mut buf=Vec::new();f.take(MAX_BYTES+1).read_to_end(&mut buf).map_err(|e|e.to_string())?;if buf.len() as u64>MAX_BYTES{return Err("Checkpoint exceeds 32 MiB".into());}let v:Self=serde_json::from_slice(&buf).map_err(|e|e.to_string())?;v.validate(entry)?;Ok(v)}
}
#[cfg(test)]mod tests{use super::*;#[test]fn checkpoint_roundtrip(){let mut p=Project::default();p.objects[0].pro.variables.insert("score".into(),42.);let s=Checkpoint{version:1,entry:"project.forge.json".into(),level:1,project:p.clone(),inventory:None,demo:None,elapsed:4.,finished:false};let path=std::env::temp_dir().join(format!("forge-save-{}.json",uuid::Uuid::new_v4()));s.save(&path).unwrap();s.save(&path).unwrap();let restored=Checkpoint::load(&path,"project.forge.json").unwrap();assert_eq!(restored.project,p);assert!(Checkpoint::load(&path,"other.json").is_err());std::fs::remove_file(path).unwrap();}}

#[cfg(test)]
mod slot_tests {
 use super::*;
 #[test] fn slot_paths_preserve_legacy_and_reject_invalid(){
  let p=Path::new("project.forge.json");
  assert_eq!(slot_path(p,1).unwrap(),Path::new("project.forge.checkpoint.json"));
  assert_eq!(slot_path(p,2).unwrap(),Path::new("project.forge.checkpoint.slot2.json"));
  assert_ne!(slot_path(p,2).unwrap(),slot_path(p,3).unwrap());
  assert!(slot_path(p,0).is_err());
  assert!(slot_path(p,4).is_err());
 }
 #[test] fn independent_slots_and_corruption(){
  let dir=std::env::temp_dir().join(format!("haver-slots-{}",uuid::Uuid::new_v4()));
  std::fs::create_dir(&dir).unwrap();
  let entry=dir.join("project.forge.json");
  let mut c=Checkpoint{version:1,entry:"project.forge.json".into(),level:1,project:Project::default(),inventory:None,demo:None,elapsed:1.,finished:false};
  let a=slot_path(&entry,1).unwrap();let b=slot_path(&entry,2).unwrap();
  c.save(&a).unwrap();c.elapsed=2.;c.save(&b).unwrap();c.elapsed=3.;c.save(&b).unwrap();
  assert_eq!(Checkpoint::load(&a,&c.entry).unwrap().elapsed,1.);
  assert_eq!(Checkpoint::load(&b,&c.entry).unwrap().elapsed,3.);
  assert!(Checkpoint::load(&slot_path(&entry,3).unwrap(),&c.entry).is_err());
  std::fs::write(&b,b"not json").unwrap();
  assert!(Checkpoint::load(&b,&c.entry).is_err());
  assert_eq!(Checkpoint::load(&a,&c.entry).unwrap().elapsed,1.);
  std::fs::remove_dir_all(dir).unwrap();
 }
}
