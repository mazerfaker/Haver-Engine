//! Opt-in shader test. Not the scene renderer; CPU fallback is unchanged.
use bevy::{prelude::*,reflect::TypePath,render::{render_resource::{AsBindGroup,ShaderRef,Extent3d,TextureDimension,TextureFormat},render_asset::RenderAssetUsages},sprite::{Material2d,Material2dPlugin}};
#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
struct LabMaterial {
 #[uniform(0)] light:Vec4,
 #[texture(1)] #[sampler(2)] normal:Handle<Image>,
}
impl Material2d for LabMaterial {fn fragment_shader()->ShaderRef{"shaders/forge_lab_light.wgsl".into()}}
#[derive(Resource)] struct Lab(Handle<LabMaterial>);
fn setup(mut commands:Commands,mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<LabMaterial>>,mut images:ResMut<Assets<Image>>){
 commands.spawn(Camera2d);
 let mut bytes=Vec::new();
 for y in 0..128 {for x in 0..128 {let dx=(x as f32/127.*2.-1.)*0.6;let dy=(y as f32/127.*2.-1.)*0.6;let z=(1.-dx*dx-dy*dy).max(0.).sqrt();for c in [dx,dy,z]{bytes.push(((c*0.5+0.5)*255.) as u8);}bytes.push(255);}}
 let normal=images.add(Image::new(Extent3d{width:128,height:128,depth_or_array_layers:1},TextureDimension::D2,bytes,TextureFormat::Rgba8Unorm,RenderAssetUsages::default()));
 let mat=materials.add(LabMaterial{light:Vec4::new(0.25,0.5,0.8,0.22),normal});
 commands.spawn((Mesh2d(meshes.add(Rectangle::new(560.,360.))),MeshMaterial2d(mat.clone()),Transform::default()));
 commands.insert_resource(Lab(mat));
}
fn animate(time:Res<Time>,lab:Res<Lab>,mut materials:ResMut<Assets<LabMaterial>>){if let Some(m)=materials.get_mut(&lab.0){let t=time.elapsed_secs();m.light.x=0.5+t.cos()*0.42;m.light.y=0.5+t.sin()*0.42;}}
fn main(){App::new().add_plugins((DefaultPlugins,Material2dPlugin::<LabMaterial>::default())).add_systems(Startup,setup).add_systems(Update,animate).run();}
