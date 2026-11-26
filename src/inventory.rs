//! Bounded, opt-in inventory. Slot changes are atomic when a pickup cannot fit.
use serde::{Serialize,Deserialize};
use std::{collections::BTreeSet,path::Path};
use uuid::Uuid;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct ItemStack{pub item:String,pub count:u32}
impl Default for ItemStack{fn default()->Self{Self{item:"Item".into(),count:1}}}
impl ItemStack{pub fn valid(&self)->bool{!self.item.trim().is_empty()&&self.item.len()<=128&&self.count>0&&self.count<=99}}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(default)]
pub struct Inventory{pub slots:Vec<Option<ItemStack>>,pub collected:BTreeSet<Uuid>}
impl Default for Inventory{fn default()->Self{Self{slots:vec![None;12],collected:BTreeSet::new()}}}
impl Inventory{
 pub fn valid(&self)->bool{(1..=64).contains(&self.slots.len())&&self.collected.len()<=10000&&self.slots.iter().flatten().all(ItemStack::valid)}
 pub fn add(&mut self,item:&ItemStack)->bool{if !self.valid()||!item.valid(){return false;}let mut next=self.slots.clone();let mut left=item.count;for stack in next.iter_mut().flatten().filter(|s|s.item==item.item){let n=(99-stack.count).min(left);stack.count+=n;left-=n;}for slot in &mut next{if left==0{break;}if slot.is_none(){*slot=Some(ItemStack{item:item.item.clone(),count:left.min(99)});left-=left.min(99);}}if left>0{return false;}self.slots=next;true}
 pub fn move_slot(&mut self,from:usize,to:usize)->bool{if !self.valid()||from==to||from>=self.slots.len()||to>=self.slots.len(){return false;}let Some(a)=self.slots[from].clone()else{return false;};if let Some(b)=self.slots[to].as_mut(){if b.item==a.item{let n=(99-b.count).min(a.count);if n==0{return false;}b.count+=n;self.slots[from]=(a.count>n).then(||ItemStack{item:a.item,count:a.count-n});return true;}}self.slots.swap(from,to);true}
 pub fn save(&self,path:&Path)->Result<(),String>{if !self.valid(){return Err("Invalid inventory".into());}let data=serde_json::to_vec_pretty(self).map_err(|e|e.to_string())?;let tmp=path.with_extension("json.tmp");std::fs::write(&tmp,data).map_err(|e|e.to_string())?;std::fs::rename(tmp,path).map_err(|e|e.to_string())}
 pub fn load(path:&Path)->Result<Self,String>{let file=std::fs::File::open(path).map_err(|e|e.to_string())?;use std::io::Read;let mut buf=Vec::new();file.take(2*1024*1024+1).read_to_end(&mut buf).map_err(|e|e.to_string())?;if buf.len()>2*1024*1024{return Err("Inventory file too large".into());}let v:Self=serde_json::from_slice(&buf).map_err(|e|e.to_string())?;if v.valid(){Ok(v)}else{Err("Invalid inventory save".into())}}
 /// Only collect independent, leaf pickup objects; avoid orphaning scene descendants.
 pub fn collect_nearby(&mut self,p:&mut crate::Project)->Result<String,String>{let pos=p.objects.iter().find(|o|o.player).ok_or("No player")?.position;let parents:BTreeSet<_>=p.objects.iter().filter_map(|o|o.parent).collect();let target=p.objects.iter().filter(|o|!o.player&&!o.camera&&o.pro.pickup.is_some()&&!self.collected.contains(&o.id)&&!parents.contains(&o.id)).filter_map(|o|{let d=(o.position[0]-pos[0]).hypot(o.position[1]-pos[1]);(d<=80.).then_some((o.id,d,o.pro.pickup.as_ref().unwrap().clone()))}).min_by(|a,b|a.1.total_cmp(&b.1));let Some((id,_,item))=target else{return Err("No pickup within 80 px".into());};if self.collected.len()>=10000||!self.add(&item){return Err("Inventory full or invalid item".into());}self.collected.insert(id);p.objects.retain(|o|o.id!=id);Ok(format!("Collected {} × {}",item.item,item.count))}
 pub fn remove_collected(&self,p:&mut crate::Project){let parents:BTreeSet<_>=p.objects.iter().filter_map(|o|o.parent).collect();p.objects.retain(|o|o.player||o.camera||parents.contains(&o.id)||o.pro.pickup.is_none()||!self.collected.contains(&o.id));}
}
#[cfg(test)]mod tests{use super::*;#[test]fn full_is_atomic(){let mut i=Inventory{slots:vec![Some(ItemStack{item:"A".into(),count:98})],..Default::default()};let before=i.clone();assert!(!i.add(&ItemStack{item:"A".into(),count:2}));assert_eq!(i,before);assert!(i.add(&ItemStack{item:"A".into(),count:1}));}
#[test]fn merge_and_swap(){let mut i=Inventory{slots:vec![Some(ItemStack{item:"A".into(),count:70}),Some(ItemStack{item:"A".into(),count:50}),None],..Default::default()};assert!(i.move_slot(0,1));assert_eq!(i.slots[1].as_ref().unwrap().count,99);assert_eq!(i.slots[0].as_ref().unwrap().count,21);assert!(i.move_slot(0,2));assert!(i.slots[0].is_none());}
#[test]fn pickup_and_save(){let mut p=crate::Project::default();let mut o=crate::Object::new("pickup",[5.,0.],[8.;2],[255;4]);o.pro.pickup=Some(ItemStack::default());let id=o.id;p.objects.push(o);let mut inv=Inventory::default();assert!(inv.collect_nearby(&mut p).is_ok());assert!(inv.collected.contains(&id));assert!(!p.objects.iter().any(|o|o.id==id));let path=std::env::temp_dir().join(format!("forge-inv-{}.json",Uuid::new_v4()));inv.save(&path).unwrap();assert_eq!(Inventory::load(&path).unwrap(),inv);std::fs::remove_file(path).unwrap();}}

#[cfg(test)]mod fixture_test{use super::*;#[test]fn inventory_scene(){let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/inventory/project.forge.json");let mut p=crate::Project::load(&path).unwrap();let mut inv=p.settings.inventory.clone().unwrap();assert!(inv.collect_nearby(&mut p).is_ok());assert_eq!(inv.slots[0].as_ref().unwrap().count,5);assert_eq!(p.objects.iter().filter(|o|o.pro.pickup.is_some()).count(),2);}}
