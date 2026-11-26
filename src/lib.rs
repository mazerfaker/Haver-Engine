use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    #[serde(default)] pub production: ProductionObject,
    #[serde(default)] pub prefab_link: Option<PrefabLink>,
    #[serde(default)] pub pro: ProComponents,
    pub id: Uuid,
    #[serde(default)] pub material: PhysicsMaterial,
    #[serde(default)] pub tile: Option<TileCell>,
    #[serde(default)] pub motion: Motion,
    pub name: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub layer: i32,
    pub color: [u8; 4],
    pub sprite: Option<String>,
    pub player: bool,
    #[serde(default)] pub parent: Option<Uuid>,
    #[serde(default)] pub body: Body,
    #[serde(default)] pub animation: Animation,
    #[serde(default)] pub script: String,
    #[serde(default)] pub graph: Graph,
    #[serde(default)] pub camera: bool,
    #[serde(default)] pub clips: std::collections::BTreeMap<String,Animation>,
    #[serde(default = "default_clip")] pub clip_name: String,
}
impl Object {
    pub fn new(name: &str, position: [f32; 2], size: [f32; 2], color: [u8; 4]) -> Self {
        Self { prefab_link:None, production: ProductionObject::default(), pro: ProComponents::default(), material: PhysicsMaterial::default(), tile: None, motion: Motion::default(), id: Uuid::new_v4(), name: name.into(), position, size, rotation: 0., layer: 0, color, sprite: None, player: false, parent: None, body: Body::default(), animation: Animation::default(), script: String::new(), graph: Graph::default(), camera: false, clips: Default::default(), clip_name: default_clip() }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project { #[serde(default)] pub settings: GameSettings, pub schema: u32, pub name: String, pub objects: Vec<Object>, #[serde(default)] pub isometric: bool }
impl Default for Project {
    fn default() -> Self {
        let mut player = Object::new("Player", [0., 0.], [40., 56.], [109, 176, 235, 255]);
        player.player = true; player.layer = 2; player.body = Body::Dynamic;
        let mut ground=Object::new("Ground", [0., -110.], [460., 32.], [80, 111, 103, 255]); ground.body=Body::Static; let mut platform=Object::new("Platform", [130., -25.], [110., 20.], [100, 130, 148, 255]); platform.body=Body::Static; Self { settings: GameSettings::default(), schema: 2, isometric: false, name: "New Project".into(), objects: vec![ground,platform,player] }
    }
}
impl Project {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut p: Self = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        p.validate_data()?;
        p.schema = 2;
        Ok(p)
    }
    pub fn validate_data(&self) -> Result<(), String> {
        let p=self;
        if p.schema != 1 && p.schema != 2 { return Err("Unsupported project schema".into()); }
        let mut ids = std::collections::HashSet::new();
        for o in &p.objects {
            if !ids.insert(o.id) || !o.position.iter().chain(o.size.iter()).chain([&o.rotation]).all(|x| x.is_finite()) || o.size.iter().any(|x| *x <= 0.) { return Err("Invalid object".into()); }
            if o.motion.events.len()>256||o.motion.events.iter().any(|e|!e.time.is_finite()||e.time<0.||e.name.len()>128||e.clip.len()>128){return Err("Invalid animation events".into());}
            for a in std::iter::once(&o.animation).chain(o.clips.values()){if a.regions.len()>16384||a.regions.iter().any(|&r|!valid_region(r)){return Err("Invalid atlas region".into());}}
            if let Some(s) = &o.sprite { if !s.starts_with("assets/") || Path::new(s).components().any(|c| !matches!(c, std::path::Component::Normal(_))) { return Err("Invalid asset path".into()); } }
        }
        for o in &p.objects {if let Some(rel)=&o.production.normal_map {if !valid_asset(rel){return Err("Invalid normal map asset path".into());}}}
        for o in &p.objects {if let Some(rel)=&o.production.particle_texture{if !valid_asset(rel){return Err("Invalid particle texture asset path".into());}}}
        if p.settings.inventory.as_ref().is_some_and(|i|!i.valid())||p.objects.iter().any(|o|o.pro.pickup.as_ref().is_some_and(|i|!i.valid())){return Err("Invalid inventory or pickup".into());}
        if p.settings.item_catalog.len()>512||p.settings.item_catalog.iter().any(|(k,v)|k.trim().is_empty()||k.len()>128||!v.valid()){return Err("Invalid item catalog".into());}
        if p.settings.inventory.as_ref().is_some_and(|i|!i.equipment_valid(&p.settings.item_catalog)){return Err("Invalid initial equipment".into());}
        Ok(())
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        fs::write(path, serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn campaign_scenes_deserialize_and_roundtrip() {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/signal_lost");
        for name in ["project.forge.json","level2.forge.json","level3.forge.json","level4.forge.json"] {
            let p=Project::load(&root.join(name)).unwrap_or_else(|e|panic!("{name}: {e}"));
            assert_eq!(p.objects.iter().filter(|o|o.player).count(),1,"{name}");
            let json=serde_json::to_vec(&p).unwrap();
            let restored:Project=serde_json::from_slice(&json).unwrap();
            assert_eq!(p,restored,"{name}");
        }
    }
    #[test] fn legacy_animation_without_looping() {
        let base=r#"{"columns":1,"rows":1,"frames":1,"fps":8}"#;
        let a:Animation=serde_json::from_str(base).unwrap();
        assert_eq!(a,Animation::default());
        let mut value:serde_json::Value=serde_json::from_str(base).unwrap();
        value["looping"]=serde_json::json!(false);
        assert!(!serde_json::from_value::<Animation>(value.clone()).unwrap().looping);
        value["looping"]=serde_json::json!("false");
        assert!(serde_json::from_value::<Animation>(value).is_err());
    }
    #[test] fn legacy_named_clip_without_looping() {
        let mut value=serde_json::to_value(Project::default()).unwrap();
        value["objects"][0]["clips"]["Idle"]=serde_json::json!({"columns":1,"rows":1,"frames":1,"fps":8});
        let restored:Project=serde_json::from_value(value).unwrap();
        assert!(restored.objects[0].clips["Idle"].looping);
    }
    #[test] fn gpu_sample_loads() {let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/gpu_scene/project.forge.json");let p=Project::load(&path).unwrap();assert!(p.settings.gpu_lighting);assert!(p.objects.iter().any(|o|o.pro.light.is_some()));let normal=p.objects.iter().find_map(|o|o.production.normal_map.as_ref()).unwrap();assert!(path.parent().unwrap().join(normal).is_file());assert_eq!(crate::modules::validate(&p).is_ok(),cfg!(feature="lighting"));}
    #[test] fn legacy_gpu_defaults() {let p=Project::default();let mut v=serde_json::to_value(&p).unwrap();v["settings"].as_object_mut().unwrap().remove("gpu_lighting");v["objects"][0]["production"].as_object_mut().unwrap().remove("normal_map");let restored:Project=serde_json::from_value(v).unwrap();assert!(!restored.settings.gpu_lighting);assert!(restored.objects[0].production.normal_map.is_none());}
    #[test] fn particle_scene_roundtrip() {let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/gpu_particles/project.forge.json");let p=Project::load(&path).unwrap();assert!(p.settings.gpu_particles);assert_eq!(p.settings.gpu_light_count,3);assert_eq!(p.objects.iter().filter(|o|o.pro.emitter.is_some()).count(),5);assert_eq!(crate::modules::validate(&p).is_ok(),cfg!(feature="lighting"));assert_eq!(p,serde_json::from_value::<Project>(serde_json::to_value(&p).unwrap()).unwrap());}
    #[test] fn legacy_particle_defaults() {let mut v=serde_json::to_value(Project::default()).unwrap();for key in ["gpu_particles","gpu_light_count","shadow_softness"]{v["settings"].as_object_mut().unwrap().remove(key);}let restored:Project=serde_json::from_value(v).unwrap();assert!(!restored.settings.gpu_particles);assert_eq!(restored.settings.gpu_light_count,1);assert_eq!(restored.settings.shadow_softness,0.);}
    #[test] fn roundtrip() { let p = Project::default(); let f = std::env::temp_dir().join(format!("{}.json", Uuid::new_v4())); p.save(&f).unwrap(); assert_eq!(p, Project::load(&f).unwrap()); fs::remove_file(f).unwrap(); }
}

mod runtime;
pub use runtime::*;

fn default_clip()->String{"Idle".into()}

mod audio_effects;
pub use audio_effects::*;
mod atlas_studio;
pub use atlas_studio::*;
mod prefab_studio;
pub use prefab_studio::*;
mod animation_studio;
pub use animation_studio::*;
mod ultimate;
pub use ultimate::*;
mod editor_core;
pub use editor_core::*;

mod asset_pipeline;
pub use asset_pipeline::*;

mod max_tools;
pub use max_tools::*;

mod gameplay;
pub use gameplay::*;

mod pro_gameplay;
pub use pro_gameplay::*;

mod production;
pub use production::*;

mod demo;
pub use demo::*;

pub mod performance;

pub mod spatial;

pub mod modules;

pub mod chunk_plan;

/// Keep HUD controls inside the viewport and scale from the design resolution.
pub fn adaptive_widget(w:&Widget,viewport:[f32;2],design:[u32;2])->[f32;4]{let scale=(viewport[0]/design[0].max(1) as f32).min(viewport[1]/design[1].max(1) as f32).max(0.);let size=[(w.size[0]*scale).max(0.).min(viewport[0]),(w.size[1]*scale).max(0.).min(viewport[1])];[(w.anchor[0]*viewport[0]).clamp(0.,(viewport[0]-size[0]).max(0.)),(w.anchor[1]*viewport[1]).clamp(0.,(viewport[1]-size[1]).max(0.)),size[0],size[1]]}
#[cfg(test)] mod adaptive_tests{use super::*;#[test]fn fits(){let w=Widget{anchor:[1.,1.],..Default::default()};let r=adaptive_widget(&w,[320.,180.],[640,360]);assert!(r[0]+r[2]<=320.&&r[1]+r[3]<=180.);}}

pub fn animation_time(o:&Object,elapsed:f32)->f32 {if o.clip_name=="Attack" {o.pro.variables.get("forge_attack_time").copied().unwrap_or(elapsed)}else{(elapsed-o.pro.variables.get("forge_clip_started").copied().unwrap_or(0.)).max(0.)}}

pub mod combat;

#[cfg(feature="lighting")] #[allow(dead_code)] pub mod gpu_scene; // ShaderType derive generates unused check helpers
#[cfg(feature="lighting")] pub mod gpu_particles;

/// Normalize explicitly edited HUD group paths to the runtime's eight-level limit.
pub fn hud_group_path(path:&str)->String {path.split('/').map(str::trim).filter(|s|!s.is_empty()).take(8).collect::<Vec<_>>().join("/")}
/// Convert a drag delta from preview points into normalized screen anchors.
pub fn move_hud_widget(w:&mut Widget,delta:[f32;2],viewport:[f32;2],design:[u32;2]) {
 if viewport.iter().any(|v|!v.is_finite()||*v<=0.)||delta.iter().any(|v|!v.is_finite()){return;}
 let rect=adaptive_widget(w,viewport,design);
 for i in 0..2 {w.anchor[i]=((rect[i]+delta[i])/viewport[i]).clamp(0.,((viewport[i]-rect[i+2])/viewport[i]).max(0.));}
}
#[cfg(test)] mod hud_studio_tests{use super::*;
 #[test] fn group_paths(){assert_eq!(hud_group_path(" / HUD // Inventory / Slots / "),"HUD/Inventory/Slots");assert_eq!(hud_group_path("1/2/3/4/5/6/7/8/9"),"1/2/3/4/5/6/7/8");}
 #[test] fn drag_stays_on_canvas(){let mut w=Widget{anchor:[0.5,0.5],size:[120.,44.],..Default::default()};move_hud_widget(&mut w,[900.,-900.],[320.,180.],[640,360]);let r=adaptive_widget(&w,[320.,180.],[640,360]);assert_eq!(r,[260.,0.,60.,22.]);}
 #[test] fn legacy_hud_mode(){let s:GameSettings=serde_json::from_str("{}").unwrap();assert!(!s.hud_canvas_groups);}
}

pub mod inventory;
pub use inventory::{Inventory,ItemStack,ItemDefinition};
pub mod save_game;
