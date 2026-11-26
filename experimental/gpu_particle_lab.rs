//! Separate opt-in compute and rendering test; no readback or CPU position updates.
use bevy::{prelude::*,reflect::TypePath,render::{extract_resource::{ExtractResource,ExtractResourcePlugin},render_asset::RenderAssets,render_graph::{self,RenderGraph,RenderLabel},render_resource::{binding_types::storage_buffer,*},renderer::{RenderContext,RenderDevice},storage::{GpuShaderStorageBuffer,ShaderStorageBuffer},Render,RenderApp,RenderSet},sprite::{Material2d,Material2dPlugin}};
const N:u32=128;
#[derive(Resource,Clone,ExtractResource)] struct ParticleBuffer(Handle<ShaderStorageBuffer>);
#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)] struct ParticleMaterial{#[storage(0,read_only)] particles:Handle<ShaderStorageBuffer>}
impl Material2d for ParticleMaterial{fn fragment_shader()->ShaderRef{"shaders/forge_particle_draw.wgsl".into()}}
fn setup(mut commands:Commands,mut buffers:ResMut<Assets<ShaderStorageBuffer>>,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<ParticleMaterial>>){
 let initial:Vec<[f32;4]>=(0..N).map(|i|[0.5,0.9,i as f32/N,0.0]).collect();
 let handle=buffers.add(ShaderStorageBuffer::from(initial));
 commands.insert_resource(ParticleBuffer(handle.clone()));commands.spawn(Camera2d);
 commands.spawn((Mesh2d(meshes.add(Rectangle::new(640.,360.))),MeshMaterial2d(mats.add(ParticleMaterial{particles:handle})),Transform::default()));
}
struct ComputePlugin;
#[derive(Debug,Hash,PartialEq,Eq,Clone,RenderLabel)] struct Simulate;
#[derive(Resource)] struct Pipeline{layout:BindGroupLayout,id:CachedComputePipelineId}
#[derive(Resource)] struct Bindings(BindGroup);
impl FromWorld for Pipeline{fn from_world(w:&mut World)->Self{
 let layout=w.resource::<RenderDevice>().create_bind_group_layout("forge_particle_layout",&BindGroupLayoutEntries::sequential(ShaderStages::COMPUTE,(storage_buffer::<Vec<Vec4>>(false),)));
 let shader=w.load_asset("shaders/forge_particle_compute.wgsl");let id=w.resource::<PipelineCache>().queue_compute_pipeline(ComputePipelineDescriptor{label:Some("forge particle simulation".into()),layout:vec![layout.clone()],push_constant_ranges:vec![],shader,shader_defs:vec![],entry_point:"simulate".into(),zero_initialize_workgroup_memory:false});Self{layout,id}
}}
fn prepare(mut commands:Commands,device:Res<RenderDevice>,pipeline:Res<Pipeline>,buffer:Option<Res<ParticleBuffer>>,gpu:Res<RenderAssets<GpuShaderStorageBuffer>>,old:Option<Res<Bindings>>){
 if old.is_some(){return;}let Some(buffer)=buffer else{return;};let Some(data)=gpu.get(&buffer.0)else{return;};let bind=device.create_bind_group("forge_particle_binding",&pipeline.layout,&BindGroupEntries::sequential((data.buffer.as_entire_buffer_binding(),)));commands.insert_resource(Bindings(bind));
}
struct SimulationNode;
impl render_graph::Node for SimulationNode{fn run(&self,_:&mut render_graph::RenderGraphContext,ctx:&mut RenderContext,w:&World)->Result<(),render_graph::NodeRunError>{
 let Some(bind)=w.get_resource::<Bindings>()else{return Ok(());};let p=w.resource::<Pipeline>();let Some(pipeline)=w.resource::<PipelineCache>().get_compute_pipeline(p.id)else{return Ok(());};let mut pass=ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor{label:Some("forge particles"),..default()});pass.set_pipeline(pipeline);pass.set_bind_group(0,&bind.0,&[]);pass.dispatch_workgroups(N.div_ceil(64),1,1);Ok(())
}}
impl Plugin for ComputePlugin{fn build(&self,app:&mut App){app.add_plugins(ExtractResourcePlugin::<ParticleBuffer>::default());let render=app.sub_app_mut(RenderApp);render.add_systems(Render,prepare.in_set(RenderSet::PrepareBindGroups));let mut graph=render.world_mut().resource_mut::<RenderGraph>();graph.add_node(Simulate,SimulationNode);graph.add_node_edge(Simulate,bevy::render::graph::CameraDriverLabel);}fn finish(&self,app:&mut App){app.sub_app_mut(RenderApp).init_resource::<Pipeline>();}}
fn main(){App::new().add_plugins((DefaultPlugins,Material2dPlugin::<ParticleMaterial>::default(),ComputePlugin)).add_systems(Startup,setup).run();}
