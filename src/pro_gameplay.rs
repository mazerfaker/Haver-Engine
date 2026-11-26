use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,HashMap};
use crate::{Project,Body,Object};
use uuid::Uuid;
#[derive(Clone,Copy,Debug,Default,PartialEq,Serialize,Deserialize)] pub enum Shape { #[default] Box, Circle, Capsule }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct PhysicsPro {pub shape:Shape,pub friction:f32,pub mass:f32,pub lock_rotation:bool}
impl Default for PhysicsPro {fn default()->Self{Self{shape:Shape::Box,friction:0.6,mass:1.,lock_rotation:true}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Light {pub radius:f32,pub intensity:f32,pub color:[f32;3]}
impl Default for Light {fn default()->Self{Self{radius:180.,intensity:1.,color:[1.,0.8,0.5]}}}
#[derive(Clone,Copy,Debug,Default,PartialEq,Serialize,Deserialize)] pub enum Effect {#[default] Fire,Smoke,Rain,Snow,Custom}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Emitter {pub effect:Effect,pub rate:f32,pub lifetime:f32,pub velocity:[f32;2],pub color:[u8;4],pub size:f32}
impl Default for Emitter {fn default()->Self{Self{effect:Effect::Fire,rate:20.,lifetime:1.,velocity:[0.,60.],color:[255,130,40,220],size:5.}}}
impl Emitter {pub fn preset(effect:Effect)->Self{let mut e=Self{effect,..Default::default()}; match effect{Effect::Smoke=>{e.color=[150,150,150,120];e.lifetime=3.;e.size=12.;},Effect::Rain=>{e.velocity=[-20.,-400.];e.color=[150,190,255,180];e.rate=70.;},Effect::Snow=>{e.velocity=[10.,-35.];e.color=[255;4];e.lifetime=4.;},_=>{}}e}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Sound {pub path:String,pub volume:f32,pub music:bool,pub looping:bool,pub spatial:bool,pub distance:f32}
impl Default for Sound {fn default()->Self{Self{path:String::new(),volume:1.,music:false,looping:false,spatial:false,distance:400.}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Widget {pub text:String,pub action:String,pub anchor:[f32;2],pub size:[f32;2],pub group:String}
impl Default for Widget {fn default()->Self{Self{text:"Button".into(),action:"jump".into(),anchor:[0.8,0.8],size:[120.,44.],group:String::new()}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct GameSettings {pub production:crate::ProductionSettings,pub rapier:bool,pub ambient:f32,pub master:f32,pub music:f32,pub sfx:f32,pub widgets:Vec<Widget>}
impl Default for GameSettings {fn default()->Self{Self{production:Default::default(),rapier:false,ambient:1.,master:1.,music:1.,sfx:1.,widgets:vec![]}}}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct ProComponents {pub combat:Option<crate::combat::Combat>,pub physics:PhysicsPro,pub light:Option<Light>,pub emitter:Option<Emitter>,pub sound:Option<Sound>,pub variables:BTreeMap<String,f32>,pub graph_positions:Vec<[f32;2]>}
/// Persistent Rapier world; topology is rebuilt only when body configuration changes.
#[cfg(feature="physics-rapier")]
use rapier2d::prelude::*;
#[cfg(feature="physics-rapier")]
#[derive(Default)] pub struct RapierWorld { bodies:RigidBodySet,colliders:ColliderSet, pipeline:PhysicsPipeline,islands:IslandManager,broad:BroadPhaseMultiSap,narrow:NarrowPhase,joints:ImpulseJointSet,multi:MultibodyJointSet,ccd:CCDSolver, ids:HashMap<Uuid,RigidBodyHandle>, signature:String, last:HashMap<Uuid,([f32;2],f32)> }
#[cfg(feature="physics-rapier")]
impl RapierWorld {
 pub fn step(&mut self,p:&mut Project,dt:f32,input:[f32;2],jumps:&[(Uuid,f32)]) {const SCALE:f32=100.;if dt<=0.||!dt.is_finite(){return;}
 let signature=format!("{:?}{:?}",p.objects.iter().filter(|o|o.body!=Body::None&&!o.camera).map(|o|(o.id,o.body,o.size,&o.pro.physics,&o.material)).collect::<Vec<_>>(),p.settings.production.joints);
 if signature!=self.signature {let old=self.ids.iter().filter_map(|(id,h)|self.bodies.get(*h).map(|b|(*id,(*b.linvel(),b.angvel())))).collect::<HashMap<_,_>>();*self=Self::default();self.signature=signature;
 for o in &p.objects {if o.camera||o.body==Body::None{continue;}let mut rb=if o.body==Body::Dynamic{RigidBodyBuilder::dynamic()}else{RigidBodyBuilder::fixed()};rb=rb.translation(vector![o.position[0]/SCALE,o.position[1]/SCALE]).rotation(o.rotation.to_radians()).gravity_scale(o.material.gravity_scale);if o.pro.physics.lock_rotation{rb=rb.lock_rotations();}if let Some((v,w))=old.get(&o.id){rb=rb.linvel(*v).angvel(*w);}let h=self.bodies.insert(rb.ccd_enabled(true).build());let x=(o.size[0]/SCALE/2.).max(0.001);let y=(o.size[1]/SCALE/2.).max(0.001);let c=match o.pro.physics.shape{Shape::Box=>ColliderBuilder::cuboid(x,y),Shape::Circle=>ColliderBuilder::ball(x.min(y)),Shape::Capsule=>ColliderBuilder::capsule_y((y-x.min(y)).max(0.),x.min(y))};self.colliders.insert_with_parent(c.friction(o.pro.physics.friction.max(0.)).restitution(o.material.restitution.clamp(0.,1.)).mass(o.pro.physics.mass.max(0.001)).sensor(o.body==Body::Trigger).build(),h,&mut self.bodies);self.ids.insert(o.id,h);}
 for j in &p.settings.production.joints{if j.a==j.b{continue;}if let (Some(a),Some(b))=(self.ids.get(&j.a),self.ids.get(&j.b)){let length=(j.length/SCALE).max(0.001);self.joints.insert(*a,*b,RopeJointBuilder::new(length),true);}}
 }
 for o in &p.objects{let Some(h)=self.ids.get(&o.id)else{continue};let b=&mut self.bodies[*h];if self.last.get(&o.id)!=Some(&(o.position,o.rotation)){b.set_translation(vector![o.position[0]/SCALE,o.position[1]/SCALE],true);b.set_rotation(Rotation::new(o.rotation.to_radians()),true);}if o.player&&o.body==Body::Dynamic{let mut v=*b.linvel();v.x=input[0]*o.material.speed/SCALE;if input[1]>0.&&v.y.abs()<0.05{v.y=o.material.jump_speed/SCALE;}if (v-*b.linvel()).norm_squared()>1e-10{b.set_linvel(v,true);}}}
 for (id,vy) in jumps{if let Some(h)=self.ids.get(id){let b=&mut self.bodies[*h];let mut v=*b.linvel();v.y=*vy/SCALE;b.set_linvel(v,true);}}
 let params=IntegrationParameters{dt:dt.min(0.033),..Default::default()};self.pipeline.step(&vector![0.,-7.2],&params,&mut self.islands,&mut self.broad,&mut self.narrow,&mut self.bodies,&mut self.colliders,&mut self.joints,&mut self.multi,&mut self.ccd,None,&(),&());for o in &mut p.objects{if let Some(h)=self.ids.get(&o.id){let b=&self.bodies[*h];o.position=[b.translation().x*SCALE,b.translation().y*SCALE];o.rotation=b.rotation().angle().to_degrees();self.last.insert(o.id,(o.position,o.rotation));}}
 }
}
#[cfg(not(feature="physics-rapier"))]
#[derive(Default)] pub struct RapierWorld;
#[cfg(not(feature="physics-rapier"))]
impl RapierWorld { pub fn step(&mut self,_p:&mut Project,_dt:f32,_input:[f32;2],_jumps:&[(Uuid,f32)]) {} }
/// Simple radial lighting with segment occlusion by static AABBs. Sprite-centre approximation.
#[cfg(feature="lighting")]
pub fn lit_color(o:&Object,p:&Project)->[u8;4]{let mut rgb=[p.settings.ambient.clamp(0.,1.);3];for source in &p.objects{if let Some(l)=&source.pro.light{let d=[o.position[0]-source.position[0],o.position[1]-source.position[1]];let dist=d[0].hypot(d[1]);if dist>=l.radius||l.radius<=0.{continue;}let blocked=crate::raycast(p,source.position,d,dist).is_some_and(|(id,t)|id!=o.id&&id!=source.id&&t<dist);if blocked{continue;}let strength=(1.-dist/l.radius)*l.intensity;for i in 0..3{rgb[i]+=strength*l.color[i];}}}let mut c=o.color;for i in 0..3{c[i]=(c[i] as f32*rgb[i].clamp(0.,1.)) as u8;}c}
#[cfg(not(feature="lighting"))]
pub fn lit_color(o:&Object,_p:&Project)->[u8;4]{o.color}
/// Four-neighbour bitmask atlas convention: N,E,S,W = 1,2,4,8.
pub fn autotile(p:&mut Project,layer:i32,grid:u32){let cells:std::collections::HashSet<_>=p.objects.iter().filter_map(|o|o.tile.as_ref()).filter(|t|t.layer==layer&&t.grid==grid).map(|t|t.cell).collect();for o in &mut p.objects{if let Some(t)=&o.tile{if t.layer!=layer||t.grid!=grid{continue;}let [x,y]=t.cell;let mask=[ [x,y+1],[x+1,y],[x,y-1],[x-1,y]].iter().enumerate().fold(0,|m,(i,c)|m|((cells.contains(c) as u32)<<i));o.animation.columns=4;o.animation.rows=4;o.animation.start=mask;o.animation.frames=1;}}}
#[cfg(test)] mod tests{use super::*;#[test]fn defaults(){let p=Project::default();assert_eq!(p.settings.ambient,1.);assert_eq!(p.objects[0].pro.physics.mass,1.);}#[test]fn mask(){let mut p=Project::default();p.objects.clear();crate::paint_tile(&mut p,[0,0],32,0,None,false);crate::paint_tile(&mut p,[1,0],32,0,None,false);autotile(&mut p,0,32);assert_eq!(p.objects[0].animation.start,2);assert_eq!(p.objects[1].animation.start,8);}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct NodeParam {pub variable:String,pub value:f32}
impl Default for NodeParam{fn default()->Self{Self{variable:"score".into(),value:1.}}}
impl crate::Graph{pub fn execute(&self,event:crate::NodeKind,input:[f32;2],vars:&mut BTreeMap<String,f32>)->Vec<crate::NodeKind>{use crate::NodeKind::*;let mut q:std::collections::VecDeque<_>=self.nodes.iter().enumerate().filter_map(|(i,n)|(*n==event).then_some(i)).collect();let mut seen=std::collections::HashSet::new();let mut actions=vec![];while let Some(i)=q.pop_front(){if !seen.insert(i){continue;}let Some(n)=self.nodes.get(i).copied()else{continue;};let param=self.params.get(&i).cloned().unwrap_or_default();match n{IfRight if input[0]<=0.=>continue,IfJump if input[1]<=0.=>continue,IfPositive if vars.get(&param.variable).copied().unwrap_or(0.)<=0.=>continue,SetVariable=>{vars.insert(param.variable,param.value);},AddVariable=>{*vars.entry(param.variable).or_default()+=param.value;},MultiplyVariable=>{*vars.entry(param.variable).or_default()*=param.value;},_=>actions.push(n)}for [a,b] in &self.edges{if *a==i{q.push_back(*b);}}}actions}}
/// Stateless visual particles, deterministic for a given emitter and time.
pub fn particles(o:&Object,time:f32)->Vec<([f32;2],f32,[u8;4])>{let Some(e)=&o.pro.emitter else{return vec![]};let rate=e.rate.clamp(1.,200.);let life=e.lifetime.clamp(0.05,10.);let n=(rate*life).ceil().min(512.) as usize;let mut out=Vec::new();for i in 0..n{let birth=(time*rate).floor()-i as f32;let age=time-birth/rate;if birth<0.||age>=life{continue;}let jitter=(birth*12.9898).sin()*20.;let pos=[o.position[0]+jitter+e.velocity[0]*age,o.position[1]+e.velocity[1]*age];let mut color=e.color;color[3]=(color[3] as f32*(1.-age/life)) as u8;out.push((pos,e.size.max(1.),color));}out}
pub fn export_bundle(path:&std::path::Path,dest:&std::path::Path)->Result<(),String>{use std::{fs,path::Path};fn copy_dir(a:&Path,b:&Path)->std::io::Result<()>{fs::create_dir_all(b)?;for entry in fs::read_dir(a)?{let entry=entry?;let ty=entry.file_type()?;if ty.is_symlink(){continue;}let dst=b.join(entry.file_name());if ty.is_dir(){copy_dir(&entry.path(),&dst)?;}else if ty.is_file(){fs::copy(entry.path(),dst)?;}}Ok(())}let p=Project::load(path)?;let root=path.parent().ok_or("No project root")?;if dest.join("project.forge.json").exists()||dest.join("assets").exists(){return Err("Choose an empty destination".into());}if root.join("assets").is_dir(){copy_dir(&root.join("assets"),&dest.join("assets")).map_err(|e|e.to_string())?;}p.save(&dest.join("project.forge.json"))}

/// Linear distance attenuation for positional SFX; music is not attenuated.
pub fn sound_attenuation(source:[f32;2],listener:[f32;2],range:f32)->f32{if !range.is_finite()||range<=0.{return 0.;}let d=(source[0]-listener[0]).hypot(source[1]-listener[1]);if !d.is_finite(){return 0.;}(1.-d/range).clamp(0.,1.)}
#[cfg(test)] mod audio_0713_tests{use super::*;#[test]fn attenuation(){assert_eq!(sound_attenuation([0.,0.],[0.,0.],100.),1.);assert_eq!(sound_attenuation([50.,0.],[0.,0.],100.),0.5);assert_eq!(sound_attenuation([101.,0.],[0.,0.],100.),0.);assert_eq!(sound_attenuation([0.,0.],[0.,0.],0.),0.);}}
