use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
#[cfg(feature="physics-rapier")] use std::collections::HashMap;
use crate::{Project,Object};
#[cfg(feature="physics-rapier")] use crate::Body;
use uuid::Uuid;
#[derive(Clone,Copy,Debug,Default,PartialEq,Serialize,Deserialize)] pub enum Shape { #[default] Box, Circle, Capsule }
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct PhysicsPro {pub shape:Shape,pub friction:f32,pub mass:f32,pub lock_rotation:bool}
impl Default for PhysicsPro {fn default()->Self{Self{shape:Shape::Box,friction:0.6,mass:1.,lock_rotation:true}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Light {pub radius:f32,pub intensity:f32,pub color:[f32;3]}
impl Default for Light {fn default()->Self{Self{radius:180.,intensity:1.,color:[1.,0.8,0.5]}}}
#[derive(Clone,Copy,Debug,Default,PartialEq,Serialize,Deserialize)] pub enum Effect {#[default] Fire,Smoke,Rain,Snow,Custom,Sparks}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Emitter {pub effect:Effect,pub rate:f32,pub lifetime:f32,pub velocity:[f32;2],pub color:[u8;4],pub size:f32,pub spread:f32}
impl Default for Emitter {fn default()->Self{Self{effect:Effect::Fire,rate:20.,lifetime:1.,velocity:[0.,60.],color:[255,130,40,220],size:5.,spread:12.}}}
impl Emitter {pub fn preset(effect:Effect)->Self{let mut e=Self{effect,..Default::default()}; match effect{Effect::Smoke=>{e.color=[150,150,150,120];e.lifetime=3.;e.size=12.;},Effect::Rain=>{e.velocity=[-20.,-400.];e.color=[150,190,255,180];e.rate=70.;e.spread=100.;},Effect::Snow=>{e.velocity=[10.,-35.];e.color=[255;4];e.lifetime=4.;e.spread=100.;},Effect::Sparks=>{e.velocity=[0.,160.];e.lifetime=0.65;e.rate=80.;e.size=3.;e.color=[255,205,75,255];e.spread=25.;},_=>{}}e}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Sound {#[serde(default,skip_serializing_if="Option::is_none")]pub custom_bus:Option<String>,#[serde(default,skip_serializing_if="Option::is_none")]pub bus:Option<AudioRoute>,#[serde(default,skip_serializing_if="Option::is_none")] pub effects:Option<crate::WavEffects>,pub path:String,pub volume:f32,pub music:bool,pub looping:bool,pub spatial:bool,pub stereo_pan:bool,pub distance:f32,pub speed:f32}
impl Default for Sound {fn default()->Self{Self{custom_bus:None,bus:None,effects:None,path:String::new(),volume:1.,music:false,looping:false,spatial:false,stereo_pan:false,distance:400.,speed:1.}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct Widget {pub text:String,pub action:String,pub anchor:[f32;2],pub size:[f32;2],pub group:String}
impl Default for Widget {fn default()->Self{Self{text:"Button".into(),action:"jump".into(),anchor:[0.8,0.8],size:[120.,44.],group:String::new()}}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct GameSettings {#[serde(default,skip_serializing_if="BTreeMap::is_empty")]pub audio_buses:BTreeMap<String,AudioBusDefinition>,pub voice:f32,pub ambience:f32,pub inventory:Option<crate::Inventory>,pub item_catalog:std::collections::BTreeMap<String,crate::ItemDefinition>,pub gpu_lighting:bool,pub gpu_particles:bool,pub gpu_particle_limit:u32,pub gpu_particle_quads:bool,pub gpu_light_count:u32,pub shadow_softness:f32,pub production:crate::ProductionSettings,pub rapier:bool,pub ambient:f32,pub master:f32,pub music:f32,pub sfx:f32,pub widgets:Vec<Widget>,pub hud_canvas_groups:bool}
impl Default for GameSettings {fn default()->Self{Self{audio_buses:BTreeMap::new(),voice:1.,ambience:1.,inventory:None,item_catalog:Default::default(),gpu_lighting:false,gpu_particles:false,gpu_particle_limit:128,gpu_particle_quads:false,gpu_light_count:1,shadow_softness:0.,production:Default::default(),rapier:false,ambient:1.,master:1.,music:1.,sfx:1.,widgets:vec![],hud_canvas_groups:false}}}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)] #[serde(default)] pub struct ProComponents {pub pickup:Option<crate::ItemStack>,pub combat:Option<crate::combat::Combat>,pub physics:PhysicsPro,pub light:Option<Light>,pub emitter:Option<Emitter>,pub sound:Option<Sound>,pub variables:BTreeMap<String,f32>,pub graph_positions:Vec<[f32;2]>}
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

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]pub enum AudioRoute{Music,Sfx,Voice,Ambience}
impl Sound{pub fn route(&self)->AudioRoute{self.bus.unwrap_or(if self.music{AudioRoute::Music}else{AudioRoute::Sfx})}pub fn gain(&self,settings:&GameSettings)->f32{if let Some(name)=&self.custom_bus{if let Some(gain)=settings.custom_bus_gain(name){return gain;}}match self.route(){AudioRoute::Music=>settings.music,AudioRoute::Sfx=>settings.sfx,AudioRoute::Voice=>settings.voice,AudioRoute::Ambience=>settings.ambience}.clamp(0.,1.)}}

#[cfg(test)]mod route28{use super::*;#[test]fn legacy_and_voice(){let mut s:Sound=serde_json::from_str(r#"{"music":true}"#).unwrap();let settings=GameSettings{music:0.4,voice:0.2,..Default::default()};assert_eq!(s.gain(&settings),0.4);s.bus=Some(AudioRoute::Voice);assert_eq!(s.gain(&settings),0.2);}}


/// Named gain groups routed to another group or a built-in bus. Master is applied by the caller.
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)] #[serde(default)]
pub struct AudioBusDefinition{pub parent:String,pub gain:f32,pub mute:bool,#[serde(default,skip_serializing_if="Option::is_none")]pub effects:Option<crate::WavEffects>}
impl Default for AudioBusDefinition{fn default()->Self{Self{parent:"Master".into(),gain:1.,mute:false,effects:None}}}
pub const BUILTIN_AUDIO_BUSES:[&str;5]=["Master","Music","SFX","Voice","Ambience"];
pub fn valid_bus_name(s:&str)->bool{!s.trim().is_empty()&&s.trim()==s&&s.len()<=64&&!s.chars().any(char::is_control)&&!BUILTIN_AUDIO_BUSES.contains(&s)}
impl GameSettings{
 pub fn custom_bus_gain<'a>(&'a self,name:&'a str)->Option<f32>{let mut name=name;let mut gain=1.;for _ in 0..=32{let built=match name{"Master"=>Some(1.),"Music"=>Some(self.music),"SFX"=>Some(self.sfx),"Voice"=>Some(self.voice),"Ambience"=>Some(self.ambience),_=>None};if let Some(v)=built{return Some(gain*v.clamp(0.,1.));}let bus=self.audio_buses.get(name)?;if !bus.gain.is_finite(){return None;}gain*=if bus.mute{0.}else{bus.gain.clamp(0.,1.)};name=&bus.parent;}None}
 pub fn valid_audio_buses(&self)->bool{self.audio_buses.len()<=32&&self.audio_buses.iter().all(|(name,b)|valid_bus_name(name)&&b.gain.is_finite()&&(0. ..=1.).contains(&b.gain)&&b.effects.is_none_or(valid_bus_effects)&&self.custom_bus_gain(name).is_some())}
}
#[cfg(test)]mod custom_bus_tests{use super::*;#[test]fn route_and_cycle(){let mut s=GameSettings{music:0.5,..Default::default()};s.audio_buses.insert("Indoor".into(),AudioBusDefinition{parent:"Music".into(),gain:0.5,mute:false,effects:None});s.audio_buses.insert("Radio".into(),AudioBusDefinition{parent:"Indoor".into(),gain:0.5,mute:false,effects:None});assert_eq!(s.custom_bus_gain("Radio"),Some(0.125));assert!(s.valid_audio_buses());let sound=Sound{custom_bus:Some("Radio".into()),..Default::default()};assert_eq!(sound.gain(&s),0.125);s.audio_buses.get_mut("Indoor").unwrap().parent="Radio".into();assert!(!s.valid_audio_buses());assert_eq!(sound.gain(&s),s.sfx);}}


fn valid_bus_effects(fx:crate::WavEffects)->bool{(-12. ..=12.).contains(&fx.low_db)&&(-12. ..=12.).contains(&fx.high_db)&&(0. ..=1.).contains(&fx.room)&&(0. ..=1.).contains(&fx.reverb)}
impl Sound {
 /// Inherited bus preset is processed per source, never advertised as a summed bus DSP.
 pub fn effective_effects(&self,settings:&GameSettings)->Option<crate::WavEffects>{
  let mut bus_fx=None;if let Some(mut name)=self.custom_bus.as_deref(){if settings.custom_bus_gain(name).is_some(){for _ in 0..32{let Some(bus)=settings.audio_buses.get(name)else{break;};if let Some(fx)=bus.effects.filter(|fx|valid_bus_effects(*fx)){bus_fx=Some(fx);break;}name=&bus.parent;}}}
  match(self.effects,bus_fx){(Some(a),Some(b))=>Some(crate::WavEffects{low_db:(a.low_db+b.low_db).clamp(-12.,12.),high_db:(a.high_db+b.high_db).clamp(-12.,12.),room:1.-(1.-a.room.clamp(0.,1.))*(1.-b.room),reverb:1.-(1.-a.reverb.clamp(0.,1.))*(1.-b.reverb)}),(a,b)=>a.or(b)}
 }
}
#[cfg(test)]mod bus33{use super::*;#[test]fn inherits_nearest_preset(){let mut s=GameSettings::default();s.audio_buses.insert("Room".into(),AudioBusDefinition{effects:Some(crate::WavEffects{reverb:0.5,..Default::default()}),..Default::default()});s.audio_buses.insert("Child".into(),AudioBusDefinition{parent:"Room".into(),..Default::default()});let sound=Sound{custom_bus:Some("Child".into()),..Default::default()};assert_eq!(sound.effective_effects(&s).unwrap().reverb,0.5);assert!(s.valid_audio_buses());}}
