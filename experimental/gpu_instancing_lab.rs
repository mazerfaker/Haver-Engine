//! Experimental hardware instancing: GPU simulation, no CPU per-particle geometry or readback.
use bevy::{prelude::*, core_pipeline::core_2d::graph::{Core2d,Node2d}, ecs::query::QueryItem, render::{extract_resource::{ExtractResource,ExtractResourcePlugin},render_asset::RenderAssets,render_graph::{self,RenderGraph,RenderLabel,RenderGraphApp,ViewNode,ViewNodeRunner},render_resource::{binding_types::storage_buffer,*},renderer::{RenderContext,RenderDevice},storage::{GpuShaderStorageBuffer,ShaderStorageBuffer},view::ViewTarget,Render,RenderApp,RenderSet}};
const N:u32=1024;
const MAX_INSTANCES:u32=100000;
fn instance_count(value:Option<&str>)->u32{value.and_then(|s|s.parse::<u32>().ok()).map(|n|n.clamp(1,MAX_INSTANCES)).unwrap_or(N)}

#[derive(Resource,Clone,ExtractResource)]struct ParticleBuffer(Handle<ShaderStorageBuffer>,u32);
#[derive(Resource)]struct Pipelines{compute_layout:BindGroupLayout,draw_layout:BindGroupLayout,compute:CachedComputePipelineId,draw:CachedRenderPipelineId}
#[derive(Resource)]struct Bindings{compute:BindGroup,draw:BindGroup,count:u32}
#[derive(Debug,Hash,PartialEq,Eq,Clone,RenderLabel)]struct Simulate;
#[derive(Debug,Hash,PartialEq,Eq,Clone,RenderLabel)]struct DrawInstances;
fn setup(mut commands:Commands,mut buffers:ResMut<Assets<ShaderStorageBuffer>>){let count=instance_count(std::env::var("HAVER_GPU_INSTANCES").ok().as_deref());println!("GPU instancing lab: {count} instances; storage payload {} bytes",u64::from(count)*16);let initial:Vec<[f32;4]>=(0..count).map(|i|[0.35+0.3*(i as f32*0.618).fract(),0.9,i as f32/count as f32,0.]).collect();commands.insert_resource(ParticleBuffer(buffers.add(ShaderStorageBuffer::from(initial)),count));commands.spawn(Camera2d);}
impl FromWorld for Pipelines{fn from_world(w:&mut World)->Self{
 let device=w.resource::<RenderDevice>();let compute_layout=device.create_bind_group_layout("forge25 compute",&BindGroupLayoutEntries::sequential(ShaderStages::COMPUTE,(storage_buffer::<Vec<Vec4>>(false),)));let draw_layout=device.create_bind_group_layout("forge25 instances",&BindGroupLayoutEntries::sequential(ShaderStages::VERTEX,(storage_buffer::<Vec<Vec4>>(true),)));
 let compute_shader=w.load_asset("shaders/forge_particle_compute.wgsl");let draw_shader=w.load_asset("shaders/forge_instanced_particles.wgsl");let cache=w.resource::<PipelineCache>();
 let compute=cache.queue_compute_pipeline(ComputePipelineDescriptor{label:Some("Forge instanced simulation".into()),layout:vec![compute_layout.clone()],push_constant_ranges:vec![],shader:compute_shader,shader_defs:vec![],entry_point:"simulate".into(),zero_initialize_workgroup_memory:false});
 let draw=w.resource_mut::<PipelineCache>().queue_render_pipeline(RenderPipelineDescriptor{label:Some("Forge hardware instancing".into()),layout:vec![draw_layout.clone()],push_constant_ranges:vec![],vertex:VertexState{shader:draw_shader.clone(),shader_defs:vec![],entry_point:"vertex".into(),buffers:vec![]},fragment:Some(FragmentState{shader:draw_shader,shader_defs:vec![],entry_point:"fragment".into(),targets:vec![Some(ColorTargetState{format:TextureFormat::bevy_default(),blend:Some(BlendState::ALPHA_BLENDING),write_mask:ColorWrites::ALL})]}),primitive:PrimitiveState::default(),depth_stencil:None,multisample:MultisampleState::default(),zero_initialize_workgroup_memory:false});Self{compute_layout,draw_layout,compute,draw}
}}
fn prepare(mut commands:Commands,device:Res<RenderDevice>,p:Res<Pipelines>,buffer:Option<Res<ParticleBuffer>>,gpu:Res<RenderAssets<GpuShaderStorageBuffer>>,old:Option<Res<Bindings>>){if old.is_some(){return;}let Some(buffer)=buffer else{return;};let Some(data)=gpu.get(&buffer.0)else{return;};let compute=device.create_bind_group("forge25 sim buffer",&p.compute_layout,&BindGroupEntries::sequential((data.buffer.as_entire_buffer_binding(),)));let draw=device.create_bind_group("forge25 draw buffer",&p.draw_layout,&BindGroupEntries::sequential((data.buffer.as_entire_buffer_binding(),)));commands.insert_resource(Bindings{compute,draw,count:buffer.1});}
struct SimulationNode;
impl render_graph::Node for SimulationNode{fn run(&self,_:&mut render_graph::RenderGraphContext,ctx:&mut RenderContext,w:&World)->Result<(),render_graph::NodeRunError>{let Some(b)=w.get_resource::<Bindings>()else{return Ok(());};let p=w.resource::<Pipelines>();let Some(pipeline)=w.resource::<PipelineCache>().get_compute_pipeline(p.compute)else{return Ok(());};let mut pass=ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor{label:Some("Forge instanced simulate"),..default()});pass.set_pipeline(pipeline);pass.set_bind_group(0,&b.compute,&[]);pass.dispatch_workgroups(b.count.div_ceil(64),1,1);Ok(())}}
#[derive(Default)]struct InstanceNode;
impl ViewNode for InstanceNode{type ViewQuery=&'static ViewTarget;fn run(&self,_:&mut render_graph::RenderGraphContext,ctx:&mut RenderContext,target:QueryItem<Self::ViewQuery>,w:&World)->Result<(),render_graph::NodeRunError>{let Some(b)=w.get_resource::<Bindings>()else{return Ok(());};let p=w.resource::<Pipelines>();let Some(pipeline)=w.resource::<PipelineCache>().get_render_pipeline(p.draw)else{return Ok(());};if w.resource::<PipelineCache>().get_compute_pipeline(p.compute).is_none(){return Ok(());}let output=target.post_process_write();let mut pass=ctx.begin_tracked_render_pass(RenderPassDescriptor{label:Some("Haver hardware instances"),color_attachments:&[Some(RenderPassColorAttachment{view:output.destination,resolve_target:None,ops:Operations::default()})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None});pass.set_render_pipeline(pipeline);pass.set_bind_group(0,&b.draw,&[]);pass.draw(0..6,0..b.count);Ok(())}}
struct Lab;
impl Plugin for Lab{fn build(&self,app:&mut App){app.add_plugins(ExtractResourcePlugin::<ParticleBuffer>::default());let render=app.sub_app_mut(RenderApp);render.add_systems(Render,prepare.in_set(RenderSet::PrepareBindGroups));render.add_render_graph_node::<ViewNodeRunner<InstanceNode>>(Core2d,DrawInstances).add_render_graph_edges(Core2d,(Node2d::Tonemapping,DrawInstances,Node2d::EndMainPassPostProcessing));let mut graph=render.world_mut().resource_mut::<RenderGraph>();graph.add_node(Simulate,SimulationNode);graph.add_node_edge(Simulate,bevy::render::graph::CameraDriverLabel);}fn finish(&self,app:&mut App){app.sub_app_mut(RenderApp).init_resource::<Pipelines>();}}
fn main(){let mut app=App::new();app.add_plugins((DefaultPlugins,Lab)).add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default()).add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default()).add_systems(Startup,setup);if std::env::var_os("FORGE_GPU_PROFILE").is_some(){app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);}if let Some(output)=std::env::var_os("HAVER_BENCHMARK_JSON"){app.insert_resource(BenchmarkCapture{warmup:120,target:600,samples:Default::default(),output:output.into(),done:false}).add_systems(Update,capture_frames);}app.run();}
#[cfg(test)]mod tests{use super::*;#[test]fn workload_bounds(){assert_eq!(instance_count(None),1024);assert_eq!(instance_count(Some("65536")),65536);assert_eq!(instance_count(Some("100000")),100000);assert_eq!(instance_count(Some("999999")),MAX_INSTANCES);assert_eq!(instance_count(Some("0")),1);assert_eq!(instance_count(Some("bad")),1024);}}


#[derive(Resource)]
struct BenchmarkCapture {
    warmup: usize,
    target: usize,
    samples: forge_engine::benchmark::FrameSamples,
    output: std::path::PathBuf,
    done: bool,
}
fn capture_frames(time: Res<Time<Real>>, mut capture: ResMut<BenchmarkCapture>, mut exit: EventWriter<AppExit>) {
    if capture.done { return; }
    if capture.warmup > 0 { capture.warmup -= 1; return; }
    if !capture.samples.record(time.delta_secs_f64() * 1000.0) { return; }
    if capture.samples.len() < capture.target { return; }
    let Some(report) = capture.samples.report() else { return; };
    capture.done = true;
    let result = serde_json::json!({
        "mode": "experimental_gpu_instancing",
        "timing": "CPU wall-clock frame interval, includes presentation waits",
        "instances_requested": instance_count(std::env::var("HAVER_GPU_INSTANCES").ok().as_deref()),
        "gpu_execution_verified": false,
        "gpu_time_ms": null,
        "vram_bytes": null,
        "frame_report": report
    });
    match serde_json::to_vec_pretty(&result).map_err(|e| e.to_string()).and_then(|bytes| std::fs::write(&capture.output, bytes).map_err(|e| e.to_string())) {
        Ok(()) => { println!("Benchmark report: {}", capture.output.display()); exit.write(AppExit::Success); }
        Err(e) => { eprintln!("Benchmark export failed: {e}"); exit.write(AppExit::error()); }
    }
}
