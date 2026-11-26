use bevy::prelude::*;
use forge_engine::Project;
use std::path::PathBuf;
#[derive(Resource)] struct Game(Project);
#[derive(Component)] struct Controlled;
fn main(){
    let path=std::env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(||PathBuf::from("project.forge.json"));
    let path=match path.canonicalize(){Ok(p)=>p,Err(e)=>{eprintln!("Project: {e}");return;}};
    let p=match Project::load(&path){Ok(p)=>p,Err(e)=>{eprintln!("{e}");return;}};
    let root=path.parent().unwrap().to_string_lossy().into_owned();
    App::new().insert_resource(Game(p.clone())).add_plugins(DefaultPlugins.set(AssetPlugin{file_path:root,..default()}).set(ImagePlugin::default_nearest()).set(WindowPlugin{primary_window:Some(Window{title:p.name,..default()}),..default()})).add_systems(Startup,setup).add_systems(Update,move_player).run();
}
fn setup(mut commands:Commands,game:Res<Game>,assets:Res<AssetServer>){
    commands.spawn(Camera2d);
    for o in &game.0.objects{
        let color=Color::srgba_u8(o.color[0],o.color[1],o.color[2],o.color[3]);
        let mut sprite=Sprite::from_color(color,Vec2::from_array(o.size));
        if let Some(path)=&o.sprite {sprite.image=assets.load(path.clone());}
        let mut entity=commands.spawn((sprite,Transform::from_xyz(o.position[0],o.position[1],o.layer as f32).with_rotation(Quat::from_rotation_z(o.rotation.to_radians()))));
        if o.player{entity.insert(Controlled);}
    }
}
fn move_player(time:Res<Time>,keys:Res<ButtonInput<KeyCode>>,mut query:Query<&mut Transform,With<Controlled>>){
    let mut d=Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW){d.y+=1.;}if keys.pressed(KeyCode::KeyS){d.y-=1.;}if keys.pressed(KeyCode::KeyA){d.x-=1.;}if keys.pressed(KeyCode::KeyD){d.x+=1.;}
    for mut t in &mut query{t.translation+=(d.normalize_or_zero()*200.*time.delta_secs()).extend(0.);}
}
