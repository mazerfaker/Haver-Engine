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
