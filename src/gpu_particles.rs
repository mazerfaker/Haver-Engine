//! Opt-in scene particle pipeline. Simulation stays in GPU storage buffers.
use bevy::{prelude::*,asset::{weak_handle,AssetId},reflect::TypePath,render::{extract_resource::{ExtractResource,ExtractResourcePlugin},render_asset::{RenderAssets,RenderAssetUsages},render_graph::{self,RenderGraph,RenderLabel},render_resource::{binding_types::storage_buffer,*},renderer::{RenderContext,RenderDevice},storage::{GpuShaderStorageBuffer,ShaderStorageBuffer},Render,RenderApp,RenderSet},sprite::{Material2d,Material2dPlugin,AlphaMode2d}};
use std::{collections::{HashMap,HashSet},path::{Path,PathBuf},sync::atomic::{AtomicUsize,Ordering}};
use crate::{Project,Emitter};
use uuid::Uuid;
const COMPUTE:Handle<Shader>=weak_handle!("df92cbe7-1a38-4bfb-b9c8-071700000001");
const DRAW:Handle<Shader>=weak_handle!("df92cbe7-1a38-4bfb-b9c8-071700000002");
pub const MAX_EMITTERS:usize=8;
pub const MAX_PARTICLES:usize=128;
const HEADER:usize=3;
const QUADS:Handle<Shader>=weak_handle!("df92cbe7-1a38-4bfb-b9c8-072000000001");
const STEP:f32=1./60.;
static SLOTS:AtomicUsize=AtomicUsize::new(0);
static EMITTERS:AtomicUsize=AtomicUsize::new(0);
pub fn stats()->(usize,usize,usize){let n=EMITTERS.load(Ordering::Relaxed);let slots=SLOTS.load(Ordering::Relaxed);(n,slots,(slots+n*HEADER)*16)}
pub fn count(e:&Emitter)->usize{(e.rate.clamp(1.,200.)*e.lifetime.clamp(0.05,10.)).ceil().clamp(1.,MAX_PARTICLES as f32) as usize}
pub fn count_with_limit(e:&Emitter,limit:u32)->usize{let v=Vec2::from_array(e.velocity).clamp(Vec2::splat(-400.),Vec2::splat(400.));let extent=v.abs()*e.lifetime.clamp(0.05,10.)+Vec2::splat(2.*e.spread.clamp(0.,128.)+4.*e.size.clamp(1.,64.)+8.);let cap=if extent.x*extent.y<=300_000.{limit.clamp(1,256)}else{limit.clamp(1,128)};(e.rate.clamp(1.,200.)*e.lifetime.clamp(0.05,10.)).ceil().clamp(1.,cap as f32) as usize}
pub fn selected(p:&Project)->HashSet<Uuid>{if !p.settings.gpu_particles||p.isometric{return HashSet::new();}p.objects.iter().filter(|o|o.pro.emitter.as_ref().is_some_and(|e|{let v=Vec2::from_array(e.velocity).clamp(Vec2::splat(-400.),Vec2::splat(400.));let extent=v.abs()*e.lifetime.clamp(0.05,10.)+Vec2::splat(2.*e.spread.clamp(0.,128.)+4.*e.size.clamp(1.,64.)+8.);extent.x*extent.y<=600_000.})&&!o.camera).take(MAX_EMITTERS).map(|o|o.id).collect()}
#[derive(Resource,Clone,Default,ExtractResource)] struct Buffers{items:Vec<(Handle<ShaderStorageBuffer>,u32)>,steps:u32}
#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)] struct ParticleMaterial{#[storage(0,read_only)] particles:Handle<ShaderStorageBuffer>,#[texture(1)] #[sampler(2)] texture:Handle<Image>}
impl Material2d for ParticleMaterial{fn fragment_shader()->ShaderRef{DRAW.into()}fn alpha_mode(&self)->AlphaMode2d{AlphaMode2d::Blend}}
#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)] struct QuadMaterial{#[storage(0,read_only)] particles:Handle<ShaderStorageBuffer>,#[texture(1)] #[sampler(2)] texture:Handle<Image>}
impl Material2d for QuadMaterial{fn vertex_shader()->ShaderRef{QUADS.into()}fn fragment_shader()->ShaderRef{QUADS.into()}fn alpha_mode(&self)->AlphaMode2d{AlphaMode2d::Blend}}
/// A bounded mesh of independent quads. No per-pixel iteration over the storage buffer.
fn quad_mesh(n:usize)->Mesh {
 let n=n.clamp(1,256);let mut positions:Vec<[f32;3]>=Vec::with_capacity(n*4);let mut uv:Vec<[f32;2]>=Vec::with_capacity(n*4);let mut indices=Vec::with_capacity(n*6);
 // CCW triangles in local XY; UV.y is downwards. Encoded x identifies the slot.
 for i in 0..n {for (x,y,u,v) in [(-0.5,-0.5,0.,1.),(0.5,-0.5,1.,1.),(0.5,0.5,1.,0.),(-0.5,0.5,0.,0.)]{positions.push([x,y,0.]);uv.push([2.*i as f32+u,v]);}let b=(i*4) as u16;indices.extend([b,b+1,b+2,b,b+2,b+3]);}
 Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default()).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,positions).with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,uv).with_inserted_indices(bevy::render::mesh::Indices::U16(indices))
}
#[derive(Debug,Hash,PartialEq,Eq,Clone,RenderLabel)] struct Simulate;
#[derive(Resource)] struct Pipeline{layout:BindGroupLayout,id:CachedComputePipelineId}
#[derive(Resource,Default)] struct Bindings(HashMap<AssetId<ShaderStorageBuffer>,BindGroup>);
impl FromWorld for Pipeline{fn from_world(w:&mut World)->Self{
 let layout=w.resource::<RenderDevice>().create_bind_group_layout("forge scene particle layout",&BindGroupLayoutEntries::sequential(ShaderStages::COMPUTE,(storage_buffer::<Vec<Vec4>>(false),)));
 let id=w.resource::<PipelineCache>().queue_compute_pipeline(ComputePipelineDescriptor{label:Some("forge scene particle simulation".into()),layout:vec![layout.clone()],push_constant_ranges:vec![],shader:COMPUTE.clone(),shader_defs:vec![],entry_point:"simulate".into(),zero_initialize_workgroup_memory:false});Self{layout,id}
}}
fn prepare(device:Res<RenderDevice>,pipeline:Res<Pipeline>,buffers:Option<Res<Buffers>>,gpu:Res<RenderAssets<GpuShaderStorageBuffer>>,mut bindings:ResMut<Bindings>){
 let Some(buffers)=buffers else{bindings.0.clear();return;};let live:HashSet<_>=buffers.items.iter().map(|(h,_)|h.id()).collect();bindings.0.retain(|id,_|live.contains(id));
 for (handle,_) in &buffers.items{if bindings.0.contains_key(&handle.id()){continue;}if let Some(data)=gpu.get(handle){let bind=device.create_bind_group("forge scene particle binding",&pipeline.layout,&BindGroupEntries::sequential((data.buffer.as_entire_buffer_binding(),)));bindings.0.insert(handle.id(),bind);}}
}
struct SimulationNode;
impl render_graph::Node for SimulationNode{fn run(&self,_:&mut render_graph::RenderGraphContext,ctx:&mut RenderContext,w:&World)->Result<(),render_graph::NodeRunError>{
 let Some(buffers)=w.get_resource::<Buffers>()else{return Ok(());};if buffers.steps==0||buffers.items.is_empty(){return Ok(());}let bindings=w.resource::<Bindings>();let p=w.resource::<Pipeline>();let Some(pipeline)=w.resource::<PipelineCache>().get_compute_pipeline(p.id)else{return Ok(());};
 let mut pass=ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor{label:Some("forge scene particles"),..default()});pass.set_pipeline(pipeline);for (handle,n) in &buffers.items{if let Some(bind)=bindings.0.get(&handle.id()){pass.set_bind_group(0,bind,&[]);for _ in 0..buffers.steps{pass.dispatch_workgroups(n.div_ceil(64),1,1);}}}Ok(())
}}
pub struct GpuParticlesPlugin;
impl Plugin for GpuParticlesPlugin{fn build(&self,app:&mut App){
 bevy::asset::load_internal_asset!(app,COMPUTE,"gpu_particles_compute.wgsl",Shader::from_wgsl);bevy::asset::load_internal_asset!(app,DRAW,"gpu_particles_draw.wgsl",Shader::from_wgsl);
 bevy::asset::load_internal_asset!(app,QUADS,"gpu_particles_quads.wgsl",Shader::from_wgsl);
 app.add_plugins((Material2dPlugin::<ParticleMaterial>::default(),Material2dPlugin::<QuadMaterial>::default(),ExtractResourcePlugin::<Buffers>::default())).init_resource::<Buffers>().init_resource::<ParticleState>();let render=app.sub_app_mut(RenderApp);render.init_resource::<Bindings>().add_systems(Render,prepare.in_set(RenderSet::PrepareBindGroups));let mut graph=render.world_mut().resource_mut::<RenderGraph>();graph.add_node(Simulate,SimulationNode);graph.add_node_edge(Simulate,bevy::render::graph::CameraDriverLabel);
 }fn finish(&self,app:&mut App){app.sub_app_mut(RenderApp).init_resource::<Pipeline>();}}
struct Visual{entity:Entity,buffer:Handle<ShaderStorageBuffer>,_material:Option<Handle<ParticleMaterial>>,_quad_material:Option<Handle<QuadMaterial>>,fast:bool,emitter:Emitter,texture:Option<PathBuf>,count:usize,extent:Vec2,offset:Vec2}
#[derive(Resource,Default)] struct ParticleState{visuals:HashMap<Uuid,Visual>,quad:Option<Handle<Mesh>>,quad_meshes:HashMap<usize,Handle<Mesh>>,white:Option<Handle<Image>>,textures:HashMap<PathBuf,Option<Handle<Image>>>,accumulator:f32}
#[derive(bevy::ecs::system::SystemParam)] pub struct ParticleDraw<'w,'s>{state:ResMut<'w,ParticleState>,list:ResMut<'w,Buffers>,buffers:ResMut<'w,Assets<ShaderStorageBuffer>>,meshes:ResMut<'w,Assets<Mesh>>,images:ResMut<'w,Assets<Image>>,materials:ResMut<'w,Assets<ParticleMaterial>>,quad_materials:ResMut<'w,Assets<QuadMaterial>>,commands:Commands<'w,'s>}
fn texture(root:&Path,rel:&str,images:&mut Assets<Image>)->Option<Handle<Image>>{if !crate::valid_asset(rel){return None;}let path=root.join(rel).canonicalize().ok()?;if !path.starts_with(root.join("assets").canonicalize().ok()?){return None;}let mut reader=image::ImageReader::open(path).ok()?;let mut limits=image::Limits::default();limits.max_image_width=Some(512);limits.max_image_height=Some(512);limits.max_alloc=Some(4*1024*1024);reader.limits(limits);let img=reader.decode().ok()?.to_rgba8();Some(images.add(Image::new(Extent3d{width:img.width(),height:img.height(),depth_or_array_layers:1},TextureDimension::D2,img.into_raw(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default())))}
impl ParticleDraw<'_,'_>{pub fn sync(&mut self,p:&Project,root:Option<&Path>,dt:f32){
 let selected=selected(p);let obsolete:Vec<_>=self.state.visuals.keys().filter(|id|!selected.contains(*id)).copied().collect();for id in obsolete{if let Some(v)=self.state.visuals.remove(&id){self.commands.entity(v.entity).despawn();}}
 if selected.is_empty(){self.list.items.clear();self.list.steps=0;self.state.accumulator=0.;self.state.textures.clear();self.state.quad_meshes.clear();SLOTS.store(0,Ordering::Relaxed);EMITTERS.store(0,Ordering::Relaxed);return;}
 self.state.accumulator+=dt.clamp(0.,0.06666667);let steps=(self.state.accumulator/STEP).floor().min(4.) as u32;self.state.accumulator-=steps as f32*STEP;self.list.steps=steps;
 let mut used=HashSet::new();for o in p.objects.iter().filter(|o|selected.contains(&o.id)){let e=o.pro.emitter.as_ref().unwrap();let key=root.zip(o.production.particle_texture.as_deref()).map(|(r,s)|r.join(s));if let Some(k)=&key{used.insert(k.clone());}
 let requested=count_with_limit(e,p.settings.gpu_particle_limit);let rebuild=self.state.visuals.get(&o.id).is_none_or(|v|v.emitter!=*e||v.texture!=key||v.count!=requested||v.fast!=p.settings.gpu_particle_quads);
 if rebuild {if let Some(v)=self.state.visuals.remove(&o.id){self.commands.entity(v.entity).despawn();}
  if self.state.quad.is_none(){self.state.quad=Some(self.meshes.add(Rectangle::new(1.,1.)));}let mesh=if p.settings.gpu_particle_quads{if !self.state.quad_meshes.contains_key(&requested){let h=self.meshes.add(quad_mesh(requested));self.state.quad_meshes.insert(requested,h);}self.state.quad_meshes[&requested].clone()}else{self.state.quad.as_ref().unwrap().clone()};
  if self.state.white.is_none(){self.state.white=Some(self.images.add(Image::new(Extent3d{width:1,height:1,depth_or_array_layers:1},TextureDimension::D2,vec![255;4],TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default())));}let white=self.state.white.as_ref().unwrap().clone();
  let tex=if let Some((r,rel))=root.zip(o.production.particle_texture.as_deref()){let k=r.join(rel);if !self.state.textures.contains_key(&k){let h=texture(r,rel,&mut self.images);self.state.textures.insert(k.clone(),h);}self.state.textures.get(&k).cloned().flatten().unwrap_or(white)}else{white};
  let n=requested;let life=e.lifetime.clamp(0.05,10.);let speed=Vec2::from_array(e.velocity).clamp(Vec2::splat(-400.),Vec2::splat(400.));let travel=speed*life;let spread=e.spread.clamp(0.,128.);let size=e.size.clamp(1.,64.);let offset=travel*0.5;let extent=travel.abs()+Vec2::splat(2.*spread+size*4.+8.);
  let mut initial=Vec::with_capacity(n+HEADER);initial.push([speed.x/extent.x,-speed.y/extent.y,life,spread/size]);initial.push(e.color.map(|x|x as f32/255.));initial.push([size/extent.x,size/extent.y,0.5-offset.x/extent.x,0.5+offset.y/extent.y]);
  for i in 0..n{let age=i as f32/n as f32*life;initial.push([initial[2][2]+initial[0][0]*age,initial[2][3]+initial[0][1]*age,age,i as f32]);}
  let buffer=self.buffers.add(ShaderStorageBuffer::from(initial));let (entity,material,quad_material)=if p.settings.gpu_particle_quads{let material=self.quad_materials.add(QuadMaterial{particles:buffer.clone(),texture:tex});(self.commands.spawn((Mesh2d(mesh),MeshMaterial2d(material.clone()),Transform::default(),bevy::render::view::NoFrustumCulling)).id(),None,Some(material))}else{let material=self.materials.add(ParticleMaterial{particles:buffer.clone(),texture:tex});(self.commands.spawn((Mesh2d(mesh),MeshMaterial2d(material.clone()),Transform::default())).id(),Some(material),None)};self.state.visuals.insert(o.id,Visual{entity,buffer,_material:material,_quad_material:quad_material,fast:p.settings.gpu_particle_quads,emitter:e.clone(),texture:key,count:n,extent,offset});
 }
 if let Some(v)=self.state.visuals.get(&o.id){let mut t=Transform::from_xyz(o.position[0]+v.offset.x,o.position[1]+v.offset.y,o.layer as f32*10.+1.);t.scale=v.extent.extend(1.);self.commands.entity(v.entity).insert(t);}
 }
 let live_counts:HashSet<_>=self.state.visuals.values().filter(|v|v.fast).map(|v|v.count).collect();self.state.quad_meshes.retain(|k,_|live_counts.contains(k));self.state.textures.retain(|k,_|used.contains(k));let items:Vec<_>=p.objects.iter().filter_map(|o|self.state.visuals.get(&o.id).map(|v|(v.buffer.clone(),v.count as u32))).collect();if self.list.items!=items{self.list.items=items;}EMITTERS.store(self.state.visuals.len(),Ordering::Relaxed);SLOTS.store(self.state.visuals.values().map(|v|v.count).sum(),Ordering::Relaxed);
}}
#[cfg(test)] mod tests{use super::*;#[test]fn bounded(){let mut p=Project::default();p.settings.gpu_particles=true;let mut e=Emitter::default();e.rate=200.;e.lifetime=10.;assert_eq!(count(&e),128);for i in 0..20{let mut o=crate::Object::new(&format!("e{i}"),[0.;2],[16.;2],[255;4]);o.pro.emitter=Some(e.clone());p.objects.push(o);}assert_eq!(selected(&p).len(),MAX_EMITTERS);p.isometric=true;assert!(selected(&p).is_empty());}}

#[cfg(test)] mod budget_tests{use super::*;#[test]fn optional_budget(){let mut e=Emitter::default();e.rate=200.;e.lifetime=2.;assert_eq!(count_with_limit(&e,128),128);assert_eq!(count_with_limit(&e,256),256);e.velocity=[400.,400.];assert_eq!(count_with_limit(&e,256),128);assert_eq!(Project::default().settings.gpu_particle_limit,128);}}

#[cfg(test)]mod index_tests{use super::*;#[test]fn indexed_quad_counts(){let m=quad_mesh(128);assert_eq!(m.count_vertices(),512);assert_eq!(m.indices().unwrap().len(),768);}}
