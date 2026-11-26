use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    #[serde(default)] pub production: ProductionObject,
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
        Self { production: ProductionObject::default(), pro: ProComponents::default(), material: PhysicsMaterial::default(), tile: None, motion: Motion::default(), id: Uuid::new_v4(), name: name.into(), position, size, rotation: 0., layer: 0, color, sprite: None, player: false, parent: None, body: Body::default(), animation: Animation::default(), script: String::new(), graph: Graph::default(), camera: false, clips: Default::default(), clip_name: default_clip() }
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
        if p.schema != 1 && p.schema != 2 { return Err("Unsupported project schema".into()); }
        let mut ids = std::collections::HashSet::new();
        for o in &p.objects {
            if !ids.insert(o.id) || !o.position.iter().chain(o.size.iter()).chain([&o.rotation]).all(|x| x.is_finite()) || o.size.iter().any(|x| *x <= 0.) { return Err("Invalid object".into()); }
            if let Some(s) = &o.sprite { if !s.starts_with("assets/") || Path::new(s).components().any(|c| !matches!(c, std::path::Component::Normal(_))) { return Err("Invalid asset path".into()); } }
        }
        p.schema = 2;
        Ok(p)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        fs::write(path, serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn roundtrip() { let p = Project::default(); let f = std::env::temp_dir().join(format!("{}.json", Uuid::new_v4())); p.save(&f).unwrap(); assert_eq!(p, Project::load(&f).unwrap()); fs::remove_file(f).unwrap(); }
}

mod runtime;
pub use runtime::*;

fn default_clip()->String{"Idle".into()}

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
