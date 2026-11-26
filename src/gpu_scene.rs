//! Opt-in Material2d scene renderer. Legacy sprite pipeline stays available.
use bevy::{prelude::*, asset::weak_handle, reflect::TypePath, render::{render_resource::{AsBindGroup,Shader,ShaderRef,ShaderType,Extent3d,TextureDimension,TextureFormat},render_asset::RenderAssetUsages},sprite::{Material2d,Material2dPlugin,AlphaMode2d}};
use std::{collections::{HashMap,HashSet},path::{Path,PathBuf}};
use crate::{Project,Object,Body,sprite_center};
const SHADER:Handle<Shader>=weak_handle!("df92cbe7-1a38-4bfb-b9c8-071600000001");
#[derive(Clone,Copy,Debug,PartialEq,ShaderType)]
struct LightParams { tint:Vec4, uv:Vec4, geometry:Vec4, rotation:Vec4, light:[Vec4;3], light_color:[Vec4;3], blockers:[Vec4;12] }
#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
struct SceneMaterial {
 #[uniform(0)] params:LightParams,
 #[texture(1)] #[sampler(2)] albedo:Handle<Image>,
 #[texture(3)] #[sampler(4)] normal:Handle<Image>,
}
impl Material2d for SceneMaterial {
 fn fragment_shader()->ShaderRef {SHADER.into()}
 fn alpha_mode(&self)->AlphaMode2d{AlphaMode2d::Blend}
}
pub struct GpuScenePlugin;
impl Plugin for GpuScenePlugin {
 fn build(&self,app:&mut App){
  bevy::asset::load_internal_asset!(app,SHADER,"gpu_scene.wgsl",Shader::from_wgsl);
  app.add_plugins(Material2dPlugin::<SceneMaterial>::default()).init_resource::<GpuScene>();
 }
}
struct Visual { entity:Entity, material:Handle<SceneMaterial>, transform:Transform }
#[derive(Resource,Default)] pub struct GpuScene { visuals:HashMap<Entity,Visual>, normals:HashMap<PathBuf,Option<Handle<Image>>>, normal_payload:HashMap<PathBuf,u64>, normal_stamps:HashMap<PathBuf,Option<(u64,std::time::SystemTime)>>, normal_poll:Option<std::time::Instant>, flat:Option<Handle<Image>>, white:Option<Handle<Image>>, quad:Option<Handle<Mesh>>, light_cache:LightGrid, light_signature:Vec<(uuid::Uuid,[u32;2],u32)>, solid_cache:crate::spatial::SpatialIndex, solid_signature:Vec<(uuid::Uuid,[u32;2],[u32;2],[u32;2])>, scratch:Vec<usize>, alive_scratch:HashSet<Entity>, normal_scratch:HashSet<PathBuf>, solid_scratch:Vec<(uuid::Uuid,Vec2,Vec2,Vec2)> }
#[derive(bevy::ecs::system::SystemParam)] pub struct GpuDraw<'w,'s> {
 state:ResMut<'w,GpuScene>, meshes:ResMut<'w,Assets<Mesh>>, images:ResMut<'w,Assets<Image>>, materials:ResMut<'w,Assets<SceneMaterial>>, commands:Commands<'w,'s>,
}
fn pixel(images:&mut Assets<Image>,rgba:[u8;4],format:TextureFormat)->Handle<Image>{images.add(Image::new(Extent3d{width:1,height:1,depth_or_array_layers:1},TextureDimension::D2,rgba.to_vec(),format,RenderAssetUsages::default()))}
fn load_normal(root:&Path,rel:&str,images:&mut Assets<Image>)->Option<(Handle<Image>,u64)>{
 if !crate::valid_asset(rel){return None;}
 let path=root.join(rel).canonicalize().ok()?; if !path.starts_with(root.join("assets").canonicalize().ok()?){return None;}
 let mut reader=image::ImageReader::open(path).ok()?;let mut limits=image::Limits::default();limits.max_image_width=Some(2048);limits.max_image_height=Some(2048);limits.max_alloc=Some(24*1024*1024);reader.limits(limits);let rgba=reader.decode().ok()?.to_rgba8();
 let bytes=u64::from(rgba.width())*u64::from(rgba.height())*4;Some((images.add(Image::new(Extent3d{width:rgba.width(),height:rgba.height(),depth_or_array_layers:1},TextureDimension::D2,rgba.into_raw(),TextureFormat::Rgba8Unorm,RenderAssetUsages::RENDER_WORLD)),bytes))
}
impl GpuDraw<'_,'_> {
 pub fn idle(&self)->bool {self.state.visuals.is_empty()}
 pub fn sync<'a>(&mut self,p:&Project,root:Option<&Path>,elapsed:f32,sources:impl Iterator<Item=(Entity,&'a Object,&'a Sprite,&'a Transform,Mut<'a,Visibility>)>){
  if (!p.settings.gpu_lighting||p.isometric)&&self.state.visuals.is_empty(){if !self.state.light_signature.is_empty()||!self.state.solid_signature.is_empty(){self.state.light_signature.clear();self.state.solid_signature.clear();self.state.light_cache=Default::default();self.state.solid_cache=Default::default();self.state.scratch=Vec::new();self.state.alive_scratch=Default::default();self.state.normal_scratch=Default::default();self.state.solid_scratch=Vec::new();}INDEX_NS.store(0,Ordering::Relaxed);SYNC_NS.store(0,Ordering::Relaxed);MATERIAL_COUNT.store(0,Ordering::Relaxed);NORMAL_PAYLOAD.store(0,Ordering::Relaxed);NORMAL_COUNT.store(0,Ordering::Relaxed);return;}
  let sync_start=std::time::Instant::now();let mut alive=std::mem::take(&mut self.state.alive_scratch);alive.clear();let mut used_normals=std::mem::take(&mut self.state.normal_scratch);used_normals.clear();
  let enabled=p.settings.gpu_lighting && !p.isometric;
  // Check only the bounded resident normal cache, not the disk per sprite/frame.
  if self.state.normal_poll.is_none_or(|last|last.elapsed()>=std::time::Duration::from_millis(500)){self.state.normal_poll=Some(std::time::Instant::now());let mut stale=Vec::new();for key in self.state.normals.keys(){if self.state.normal_stamps.get(key).copied()!=Some(normal_stamp(key)){stale.push(key.clone());}}for key in stale{if let Some(Some(h))=self.state.normals.remove(&key){self.images.remove(h.id());}self.state.normal_payload.remove(&key);self.state.normal_stamps.remove(&key);}}

  let indexing_start=std::time::Instant::now();let lights:Vec<_>=if enabled {p.objects.iter().filter_map(|o|o.pro.light.as_ref().filter(|l|l.radius>0.).map(|l|(o,l))).collect()}else{vec![]};
  let light_key=|(o,l):&(&Object,&crate::Light)|(o.id,o.position.map(f32::to_bits),l.radius.to_bits());
  if !self.state.light_signature.iter().copied().eq(lights.iter().map(light_key)){self.state.light_cache=LightGrid::new(&lights);self.state.light_signature.clear();self.state.light_signature.extend(lights.iter().map(light_key));LIGHT_REBUILDS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);}else{LIGHT_HITS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);}
  let mut solids=std::mem::take(&mut self.state.solid_scratch);solids.clear();if enabled {solids.extend(p.objects.iter().filter(|o|o.body==Body::Static && !o.camera).map(|s|{let pos=Vec2::from_array(sprite_center(s,false));let half=Vec2::from_array(s.size)*0.5;let (sn,cs)=s.rotation.to_radians().sin_cos();let half=Vec2::new(cs.abs()*half.x+sn.abs()*half.y,sn.abs()*half.x+cs.abs()*half.y);(s.id,pos,pos-half,pos+half)}));}
  let solid_key=|(id,pos,a,b):&(uuid::Uuid,Vec2,Vec2,Vec2)|(*id,pos.to_array().map(f32::to_bits),a.to_array().map(f32::to_bits),b.to_array().map(f32::to_bits));
  let indexed=(32..=4096).contains(&solids.len());if !self.state.solid_signature.iter().copied().eq(solids.iter().map(solid_key)){let mut solid_grid=crate::spatial::SpatialIndex::default();if indexed{for (i,(_,pos,a,b)) in solids.iter().enumerate(){solid_grid.insert_bounded(i,pos.to_array(),((*b-*a)*0.5).to_array(),LIGHT_CELL);}}self.state.solid_cache=solid_grid;self.state.solid_signature.clear();self.state.solid_signature.extend(solids.iter().map(solid_key));SOLID_REBUILDS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);}else{SOLID_HITS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);}let mut candidates=std::mem::take(&mut self.state.scratch);INDEX_NS.store(indexing_start.elapsed().as_nanos().min(u64::MAX as u128) as u64,Ordering::Relaxed);
  for (entity,o,sprite,transform,mut visibility) in sources {
   // No texture substitution until original image is ready. CPU fallback remains visible.
   let ready=o.sprite.is_none()||self.images.get(&sprite.image).is_some();
   if !enabled||!ready||o.camera {if self.state.visuals.contains_key(&entity){*visibility=Visibility::Inherited;}continue;}
   alive.insert(entity);
   if self.state.white.is_none(){self.state.white=Some(pixel(&mut self.images,[255;4],TextureFormat::Rgba8UnormSrgb));}let white=self.state.white.as_ref().unwrap().clone();
   if self.state.flat.is_none(){self.state.flat=Some(pixel(&mut self.images,[128,128,255,255],TextureFormat::Rgba8Unorm));}let flat=self.state.flat.as_ref().unwrap().clone();
   if self.state.quad.is_none(){self.state.quad=Some(self.meshes.add(Rectangle::new(1.,1.)));}let quad=self.state.quad.as_ref().unwrap().clone();
   let albedo=if o.sprite.is_some(){sprite.image.clone()}else{white};
   let anim_time=crate::animation_time(o,elapsed);let uv=if o.sprite.is_some(){o.animation.uv(anim_time)}else{[[0.,0.],[1.,0.],[1.,1.],[0.,1.]]};
   let (center,frame_size)=crate::sprite_frame_geometry(o,false,anim_time);let center=Vec2::from_array(center);let (sin,cos)=o.rotation.to_radians().sin_cos();
   let mut tint=o.color;tint[3]=crate::sprite_visual_alpha(o,anim_time,tint[3]);let mut params=LightParams{tint:Vec4::from_array(tint.map(|x|x as f32/255.)),uv:Vec4::new(uv[0][0],uv[0][1],uv[2][0],uv[2][1]),geometry:Vec4::new(center.x,center.y,frame_size[0],frame_size[1]),rotation:Vec4::new(cos,sin,p.settings.ambient.clamp(0.,1.),p.settings.shadow_softness.clamp(0.,32.)),light:[Vec4::ZERO;3],light_color:[Vec4::ZERO;3],blockers:[Vec4::ZERO;12]};
   let strength=|(s,l):&(&Object,&crate::Light)|{if tint[3]==0{return 0.;}let d2=center.distance_squared(Vec2::from_array(s.position));if d2>=l.radius*l.radius{return 0.;}(1.-d2.sqrt()/l.radius).max(0.)*l.intensity.max(0.)};
   let relevant=best_n_fixed::<_,3>(self.state.light_cache.candidates(center.to_array(),lights.len()).take(if tint[3]==0{0}else{usize::MAX}).map(|i|lights[i]).filter_map(|x|{let s=strength(&x);(s>0.).then_some((-s,x))}),p.settings.gpu_light_count.clamp(1,3) as usize);for (li,(_,(source,light))) in relevant.into_iter().flatten().enumerate(){
    params.light[li]=Vec4::new(source.position[0],source.position[1],light.radius,light.intensity);params.light_color[li]=Vec4::new(light.color[0],light.color[1],light.color[2],1.);
    let lp=Vec2::from_array(source.position);let margin=Vec2::from_array(o.size).max(Vec2::splat(frame_size[0].hypot(frame_size[1])))+Vec2::splat(p.settings.shadow_softness.clamp(0.,32.));let lo=center.min(lp)-margin;let hi=center.max(lp)+margin;let local=indexed&&self.state.solid_cache.query_into(((lo+hi)*0.5).to_array(),((hi-lo)*0.5).to_array(),LIGHT_CELL,&mut candidates);let ids=(0..if local{0}else{solids.len()}).chain(candidates.iter().copied().filter(|_|local));let blocks=best_n_fixed::<_,4>(ids.map(|i|solids[i]).filter(|(id,_,_,_)|*id!=o.id&&*id!=source.id).filter_map(|(_,pos,a,b)|{(a.x<=hi.x&&a.y<=hi.y&&b.x>=lo.x&&b.y>=lo.y).then_some((pos.distance_squared(center),Vec4::new(a.x,a.y,b.x,b.y)))}),4);
    for (i,(_,b)) in blocks.into_iter().flatten().enumerate(){params.blockers[li*4+i]=b;}
   }
   let normal=if lights.is_empty(){flat.clone()}else if params.light.iter().all(|l|l.z<=0.){flat.clone()}else if let Some((root,rel))=root.zip(o.production.normal_map.as_deref()) {
    let key=root.join(rel);used_normals.insert(key.clone());
    // Limit resident distinct normal maps; oversized or unavailable maps use flat normal.
    if !self.state.normals.contains_key(&key)&&self.state.normals.len()<8 {let loaded=load_normal(root,rel,&mut self.images);let handle=loaded.map(|(handle,bytes)|{self.state.normal_payload.insert(key.clone(),bytes);handle});self.state.normal_stamps.insert(key.clone(),normal_stamp(&key));self.state.normals.insert(key.clone(),handle);}
    self.state.normals.get(&key).cloned().flatten().unwrap_or(flat.clone())
   }else{flat.clone()};
   // The unused fourth color component tells the shader whether a real normal texture is bound.
   params.light_color[0].w=if normal==flat{0.}else{1.};
   let mut t=*transform;t.translation.x=center.x;t.translation.y=center.y;t.scale=Vec3::new(frame_size[0],frame_size[1],1.);
   if let Some(v)=self.state.visuals.get(&entity){if self.materials.get(&v.material).is_some_and(|m|m.params!=params||m.albedo!=albedo||m.normal!=normal){if let Some(m)=self.materials.get_mut(&v.material){m.params=params;m.albedo=albedo;m.normal=normal;}}if v.transform!=t{self.commands.entity(v.entity).insert(t);if let Some(v)=self.state.visuals.get_mut(&entity){v.transform=t;}}}
   else {let material=self.materials.add(SceneMaterial{params,albedo,normal});let visual=self.commands.spawn((Mesh2d(quad),MeshMaterial2d(material.clone()),t)).id();self.state.visuals.insert(entity,Visual{entity:visual,material,transform:t});}
   if *visibility!=Visibility::Hidden{*visibility=Visibility::Hidden;}
  }
  let commands=&mut self.commands;let materials=&mut self.materials;self.state.visuals.retain(|id,v|{if alive.contains(id){true}else{commands.entity(v.entity).despawn();materials.remove(v.material.id());false}});
  let images=&mut self.images;self.state.normals.retain(|p,handle|{if used_normals.contains(p){true}else{if let Some(h)=handle{images.remove(h.id());}false}});self.state.normal_payload.retain(|p,_|used_normals.contains(p));self.state.normal_stamps.retain(|p,_|used_normals.contains(p));NORMAL_PAYLOAD.store(self.state.normal_payload.values().copied().sum(),Ordering::Relaxed);NORMAL_COUNT.store(self.state.normal_payload.len() as u64,Ordering::Relaxed);self.state.scratch=candidates;self.state.alive_scratch=alive;self.state.normal_scratch=used_normals;self.state.solid_scratch=solids;MATERIAL_COUNT.store(self.state.visuals.len() as u64,Ordering::Relaxed);SYNC_NS.store(sync_start.elapsed().as_nanos().min(u64::MAX as u128) as u64,Ordering::Relaxed);
 }
}

/// Keep the same stable distance ordering as a full sort, but retain only the requested candidates.
#[cfg(test)]
fn best_n<T>(items:impl Iterator<Item=(f32,T)>,n:usize)->Vec<(f32,T)>{let mut best:Vec<(f32,T)>=Vec::with_capacity(n);if n==0{return best;}for item in items{let at=best.partition_point(|v|v.0.total_cmp(&item.0).is_le());if at<n{if best.len()==n{best.pop();}best.insert(at,item);}}best}
#[cfg(test)]mod ranking_tests{use super::*;#[test]fn stable_top_four(){let a=[(4_f32,0),(1.,1),(1.,2),(3.,3),(2.,4),(0.,5)];let mut all=a.to_vec();all.sort_by(|a,b|a.0.total_cmp(&b.0));assert_eq!(best_n(a.into_iter(),4),all[..4]);}}

/// Point queries for light influence bounds. Large lights stay in a bounded global list.
const LIGHT_CELL:f32=256.;
#[derive(Default)]struct LightGrid{cells:HashMap<(i32,i32),Vec<usize>>,global:Vec<usize>,enabled:bool}
impl LightGrid{
 fn new(lights:&[(&Object,&crate::Light)])->Self{let mut grid=Self::default();if lights.len()<16||lights.len()>4096{return grid;}grid.enabled=true;for (i,(o,l)) in lights.iter().enumerate(){if let Some((lo,hi))=crate::spatial::cells(o.position,[l.radius;2],LIGHT_CELL){if (0..2).all(|a|i64::from(hi[a])-i64::from(lo[a])<=8){for x in lo[0]..=hi[0]{for y in lo[1]..=hi[1]{grid.cells.entry((x,y)).or_default().push(i);}}continue;}}grid.global.push(i);}grid}
 fn candidates(&self,p:[f32;2],count:usize)->impl Iterator<Item=usize>+'_ {let full=!self.enabled||p.iter().any(|v|!v.is_finite());let key=((p[0]/LIGHT_CELL).floor() as i32,(p[1]/LIGHT_CELL).floor() as i32);let local=if full{&[][..]}else{self.cells.get(&key).map(Vec::as_slice).unwrap_or(&[])};let global=if full{&[][..]}else{self.global.as_slice()};let(mut a,mut b)=(0,0);(0..if full{count}else{0}).chain(std::iter::from_fn(move||{match(local.get(a),global.get(b)){(Some(&x),Some(&y)) if x<y=>{a+=1;Some(x)},(_,Some(&y))=>{b+=1;Some(y)},(Some(&x),None)=>{a+=1;Some(x)},_=>None}}))}
}
#[cfg(test)]mod grid_tests{use super::*;#[test]fn conservative_and_stable(){let mut o:Vec<_>=(0..20).map(|i|Object::new("light",[i as f32*500.,0.],[1.;2],[255;4])).collect();for x in &mut o{x.pro.light=Some(crate::Light{radius:300.,..Default::default()});}o[5].pro.light.as_mut().unwrap().radius=100000.;let lights:Vec<_>=o.iter().map(|x|(x,x.pro.light.as_ref().unwrap())).collect();let grid=LightGrid::new(&lights);for p in [[0.,0.],[510.,0.],[-200.,0.],[4000.,0.]]{let hits:Vec<_>=grid.candidates(p,lights.len()).collect();assert!(hits.windows(2).all(|w|w[0]<w[1]));for(i,(x,l))in lights.iter().enumerate(){if Vec2::from_array(p).distance(Vec2::from_array(x.position))<l.radius{assert!(hits.contains(&i));}}}}}


use std::sync::atomic::{AtomicU64,Ordering};
static SYNC_NS:AtomicU64=AtomicU64::new(0);
pub fn last_sync_cpu_ns()->u64{SYNC_NS.load(Ordering::Relaxed)}
static INDEX_NS:AtomicU64=AtomicU64::new(0);
pub fn last_index_cpu_ns()->u64{INDEX_NS.load(Ordering::Relaxed)}
static LIGHT_REBUILDS:AtomicU64=AtomicU64::new(0);static LIGHT_HITS:AtomicU64=AtomicU64::new(0);static SOLID_REBUILDS:AtomicU64=AtomicU64::new(0);static SOLID_HITS:AtomicU64=AtomicU64::new(0);
/// Process-wide broad phase cache counters; no GPU timestamps implied.
pub fn index_cache_stats()->[u64;4]{[LIGHT_REBUILDS.load(Ordering::Relaxed),LIGHT_HITS.load(Ordering::Relaxed),SOLID_REBUILDS.load(Ordering::Relaxed),SOLID_HITS.load(Ordering::Relaxed)]}


/// Fixed-size stable ranking; no heap allocation for small per-sprite candidate sets.
fn best_n_fixed<T,const N:usize>(items:impl Iterator<Item=(f32,T)>,n:usize)->[Option<(f32,T)>;N]{
 let mut best:[Option<(f32,T)>;N]=std::array::from_fn(|_|None);let n=n.min(N);if n==0{return best;}
 for item in items{let at=best[..n].iter().position(|v|v.as_ref().is_none_or(|v|v.0.total_cmp(&item.0).is_gt())).unwrap_or(n);if at<n{for i in (at+1..n).rev(){best.swap(i,i-1);}best[at]=Some(item);}}
 best
}
#[cfg(test)]mod fixed_ranking33{use super::*;#[test]fn matches_previous_stable_order(){let a=[(4_f32,0),(1.,1),(1.,2),(3.,3),(2.,4),(0.,5)];let fixed:Vec<_>=best_n_fixed::<_,4>(a.into_iter(),4).into_iter().flatten().collect();assert_eq!(fixed,best_n(a.into_iter(),4));}}

static MATERIAL_COUNT:AtomicU64=AtomicU64::new(0);
/// Number of owned scene-lighting materials (not total GPU allocations).
pub fn lighting_material_count()->u64{MATERIAL_COUNT.load(Ordering::Relaxed)}

static NORMAL_PAYLOAD:AtomicU64=AtomicU64::new(0);static NORMAL_COUNT:AtomicU64=AtomicU64::new(0);
/// RGBA8 texel payload of owned normal maps. Not device allocation size or total VRAM.
pub fn normal_map_payload()->(u64,u64){(NORMAL_COUNT.load(Ordering::Relaxed),NORMAL_PAYLOAD.load(Ordering::Relaxed))}

fn normal_stamp(p:&Path)->Option<(u64,std::time::SystemTime)>{let m=std::fs::metadata(p).ok()?;if !m.is_file(){return None;}Some((m.len(),m.modified().ok()?))}
