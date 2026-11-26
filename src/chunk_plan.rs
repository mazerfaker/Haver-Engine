//! Conservative render planning. Does not merge draw calls yet.
use crate::{Project,Body};
use std::collections::BTreeMap;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkKey { pub layer:i32,pub grid:u32,pub x:i32,pub y:i32,pub texture:Option<String> }
/// Group only static, non-scripted, single-frame, non-parented tiles.
/// Keep original entities available for physics and editing.
pub fn candidates(p:&Project)->BTreeMap<ChunkKey,Vec<Uuid>> {
 let mut out=BTreeMap::<ChunkKey,Vec<Uuid>>::new();
 if p.isometric {return out;}
 for o in &p.objects {
  let Some(t)=&o.tile else {continue};
  if o.camera || o.player || o.parent.is_some() || o.body==Body::Dynamic || o.body==Body::Trigger || !o.script.trim().is_empty() || o.animation.frames!=1 || !o.clips.is_empty() || !o.motion.keys.is_empty() || !o.motion.clip_keys.is_empty() || o.pro.light.is_some() || o.pro.emitter.is_some() || o.production.navigation.is_some() {continue;}
  out.entry(ChunkKey{layer:t.layer,grid:t.grid,x:t.cell[0].div_euclid(16),y:t.cell[1].div_euclid(16),texture:o.sprite.clone()}).or_default().push(o.id);
 }
 out
}
