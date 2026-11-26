use super::*;
use bevy::audio::{AudioSource,AudioPlayer,PlaybackSettings};
#[derive(Component)] pub(super) struct PreviewVoice;
#[derive(Resource,Default)] pub(super) struct PreviewRequest(pub Option<Option<(PathBuf,f32)>>);
pub(super) fn process(mut request:ResMut<PreviewRequest>,mut commands:Commands,mut assets:ResMut<Assets<AudioSource>>,voices:Query<Entity,With<PreviewVoice>>,mut editor:ResMut<Editor>){
 let Some(request)=request.0.take()else{return;};for entity in &voices{commands.entity(entity).despawn();}
 let Some((path,volume))=request else{return;};
 let result=(||->Result<Vec<u8>,String>{let m=std::fs::metadata(&path).map_err(|e|e.to_string())?;if m.len()>32*1024*1024{return Err("Preview limit: 32 MiB".into());}std::fs::read(path).map_err(|e|e.to_string())})();
 match result{Ok(bytes)=>{let handle=assets.add(AudioSource{bytes:bytes.into()});commands.spawn((PreviewVoice,AudioPlayer::new(handle),PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume.clamp(0.,1.)))));editor.status="Audio preview requested".into();},Err(err)=>editor.status=err}
}
