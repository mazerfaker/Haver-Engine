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
pub(super) fn effects_menu(ui:&mut egui::Ui,e:&mut Editor){ui.collapsing("Offline WAV effects",|ui|{
 let key=egui::Id::new("forge_wav_fx_0720");let mut fx=ui.ctx().data(|d|d.get_temp::<forge_engine::WavEffects>(key)).unwrap_or_default();ui.add(egui::Slider::new(&mut fx.low_db,-12. ..=12.).text("Low dB"));ui.add(egui::Slider::new(&mut fx.high_db,-12. ..=12.).text("High dB"));ui.add(egui::Slider::new(&mut fx.room,0. ..=1.).text("Room reflections"));ui.ctx().data_mut(|d|d.insert_temp(key,fx));ui.label("PCM16 WAV only; creates a new asset, original unchanged.");
 if ui.add_enabled(e.play.is_none(),egui::Button::new("Render WAV effects…")).clicked(){if let Some(root)=e.path.as_ref().and_then(|p|p.parent()){if let Some(file)=rfd::FileDialog::new().add_filter("PCM16 WAV",&["wav"]).pick_file(){match forge_engine::render_wav_effects(&file,root,fx){Ok(rel)=>{if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){o.pro.sound.get_or_insert_with(Default::default).path=rel.clone();}e.status=format!("Rendered {rel}");},Err(err)=>e.status=err}}}else{e.status="Save project first".into();}}
 });}
