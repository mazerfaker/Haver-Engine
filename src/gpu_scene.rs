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
#[derive(Resource,Default)] pub struct GpuScene { visuals:HashMap<Entity,Visual>, normals:HashMap<PathBuf,Option<Handle<Image>>>, flat:Option<Handle<Image>>, white:Option<Handle<Image>>, quad:Option<Handle<Mesh>> }
#[derive(bevy::ecs::system::SystemParam)] pub struct GpuDraw<'w,'s> {
 state:ResMut<'w,GpuScene>, meshes:ResMut<'w,Assets<Mesh>>, images:ResMut<'w,Assets<Image>>, materials:ResMut<'w,Assets<SceneMaterial>>, commands:Commands<'w,'s>,
}
fn pixel(images:&mut Assets<Image>,rgba:[u8;4],format:TextureFormat)->Handle<Image>{images.add(Image::new(Extent3d{width:1,height:1,depth_or_array_layers:1},TextureDimension::D2,rgba.to_vec(),format,RenderAssetUsages::default()))}
fn load_normal(root:&Path,rel:&str,images:&mut Assets<Image>)->Option<Handle<Image>>{
 if !crate::valid_asset(rel){return None;}
 let path=root.join(rel).canonicalize().ok()?; if !path.starts_with(root.join("assets").canonicalize().ok()?){return None;}
 let mut reader=image::ImageReader::open(path).ok()?;let mut limits=image::Limits::default();limits.max_image_width=Some(2048);limits.max_image_height=Some(2048);limits.max_alloc=Some(24*1024*1024);reader.limits(limits);let rgba=reader.decode().ok()?.to_rgba8();
 Some(images.add(Image::new(Extent3d{width:rgba.width(),height:rgba.height(),depth_or_array_layers:1},TextureDimension::D2,rgba.into_raw(),TextureFormat::Rgba8Unorm,RenderAssetUsages::default())))
}
impl GpuDraw<'_,'_> {
 pub fn idle(&self)->bool {self.state.visuals.is_empty()}
 pub fn sync<'a>(&mut self,p:&Project,root:Option<&Path>,elapsed:f32,sources:impl Iterator<Item=(Entity,&'a Object,&'a Sprite,&'a Transform,Mut<'a,Visibility>)>){
  if (!p.settings.gpu_lighting||p.isometric)&&self.state.visuals.is_empty(){return;}
  let mut alive=HashSet::new();let mut used_normals=HashSet::new();
  let enabled=p.settings.gpu_lighting && !p.isometric;
  let lights:Vec<_>=if enabled {p.objects.iter().filter_map(|o|o.pro.light.as_ref().filter(|l|l.radius>0.).map(|l|(o,l))).collect()}else{vec![]};
  let light_grid=LightGrid::new(&lights);
  let solids:Vec<_>=if enabled {p.objects.iter().filter(|o|o.body==Body::Static && !o.camera).map(|s|{let pos=Vec2::from_array(sprite_center(s,false));let half=Vec2::from_array(s.size)*0.5;let (sn,cs)=s.rotation.to_radians().sin_cos();let half=Vec2::new(cs.abs()*half.x+sn.abs()*half.y,sn.abs()*half.x+cs.abs()*half.y);(s.id,pos,pos-half,pos+half)}).collect()}else{vec![]};
  for (entity,o,sprite,transform,mut visibility) in sources {
   // No texture substitution until original image is ready. CPU fallback remains visible.
   let ready=o.sprite.is_none()||self.images.get(&sprite.image).is_some();
   if !enabled||!ready||o.camera {if self.state.visuals.contains_key(&entity){*visibility=Visibility::Inherited;}continue;}
   alive.insert(entity);
   if self.state.white.is_none(){self.state.white=Some(pixel(&mut self.images,[255;4],TextureFormat::Rgba8UnormSrgb));}let white=self.state.white.as_ref().unwrap().clone();
   if self.state.flat.is_none(){self.state.flat=Some(pixel(&mut self.images,[128,128,255,255],TextureFormat::Rgba8Unorm));}let flat=self.state.flat.as_ref().unwrap().clone();
   if self.state.quad.is_none(){self.state.quad=Some(self.meshes.add(Rectangle::new(1.,1.)));}let quad=self.state.quad.as_ref().unwrap().clone();
   let normal=if let Some((root,rel))=root.zip(o.production.normal_map.as_deref()) {
    let key=root.join(rel);used_normals.insert(key.clone());
    // Limit resident distinct normal maps; oversized or unavailable maps use flat normal.
    if !self.state.normals.contains_key(&key)&&self.state.normals.len()<8 {let loaded=load_normal(root,rel,&mut self.images);self.state.normals.insert(key.clone(),loaded);}
    self.state.normals.get(&key).cloned().flatten().unwrap_or(flat)
   }else{flat};
   let albedo=if o.sprite.is_some(){sprite.image.clone()}else{white};
   let uv=if o.sprite.is_some(){o.animation.uv(crate::animation_time(o,elapsed))}else{[[0.,0.],[1.,0.],[1.,1.],[0.,1.]]};
   let (center,frame_size)=crate::sprite_frame_geometry(o,false,crate::animation_time(o,elapsed));let center=Vec2::from_array(center);let (sin,cos)=o.rotation.to_radians().sin_cos();
   let mut params=LightParams{tint:Vec4::from_array(o.color.map(|x|x as f32/255.)),uv:Vec4::new(uv[0][0],uv[0][1],uv[2][0],uv[2][1]),geometry:Vec4::new(center.x,center.y,frame_size[0],frame_size[1]),rotation:Vec4::new(cos,sin,p.settings.ambient.clamp(0.,1.),p.settings.shadow_softness.clamp(0.,32.)),light:[Vec4::ZERO;3],light_color:[Vec4::ZERO;3],blockers:[Vec4::ZERO;12]};
   let strength=|(s,l):&(&Object,&crate::Light)|{let d2=center.distance_squared(Vec2::from_array(s.position));if d2>=l.radius*l.radius{return 0.;}(1.-d2.sqrt()/l.radius).max(0.)*l.intensity.max(0.)};
   let relevant=best_n(light_grid.candidates(center.to_array(),lights.len()).map(|i|lights[i]).filter_map(|x|{let s=strength(&x);(s>0.).then_some((-s,x))}),p.settings.gpu_light_count.clamp(1,3) as usize);for (li,(_,(source,light))) in relevant.into_iter().enumerate(){
    params.light[li]=Vec4::new(source.position[0],source.position[1],light.radius,light.intensity);params.light_color[li]=Vec4::new(light.color[0],light.color[1],light.color[2],1.);
    let lp=Vec2::from_array(source.position);let margin=Vec2::from_array(o.size);let lo=center.min(lp)-margin;let hi=center.max(lp)+margin;let blocks=best_n(solids.iter().copied().filter(|(id,_,_,_)|*id!=o.id&&*id!=source.id).filter_map(|(_,pos,a,b)|{(a.x<=hi.x&&a.y<=hi.y&&b.x>=lo.x&&b.y>=lo.y).then_some((pos.distance_squared(center),Vec4::new(a.x,a.y,b.x,b.y)))}),4);
    for (i,(_,b)) in blocks.into_iter().enumerate(){params.blockers[li*4+i]=b;}
   }
   let mut t=*transform;t.translation.x=center.x;t.translation.y=center.y;t.scale=Vec3::new(frame_size[0],frame_size[1],1.);
   if let Some(v)=self.state.visuals.get(&entity){if self.materials.get(&v.material).is_some_and(|m|m.params!=params||m.albedo!=albedo||m.normal!=normal){if let Some(m)=self.materials.get_mut(&v.material){m.params=params;m.albedo=albedo;m.normal=normal;}}if v.transform!=t{self.commands.entity(v.entity).insert(t);if let Some(v)=self.state.visuals.get_mut(&entity){v.transform=t;}}}
   else {let material=self.materials.add(SceneMaterial{params,albedo,normal});let visual=self.commands.spawn((Mesh2d(quad),MeshMaterial2d(material.clone()),t)).id();self.state.visuals.insert(entity,Visual{entity:visual,material,transform:t});}
   if *visibility!=Visibility::Hidden{*visibility=Visibility::Hidden;}
  }
  let obsolete:Vec<_>=self.state.visuals.keys().copied().filter(|id|!alive.contains(id)).collect();for id in obsolete{if let Some(v)=self.state.visuals.remove(&id){self.commands.entity(v.entity).despawn();}}
  self.state.normals.retain(|p,_|used_normals.contains(p));
 }
}

/// Keep the same stable distance ordering as a full sort, but retain only the requested candidates.
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
