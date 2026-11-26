use super::*;
use bevy::audio::{AudioSource,AudioPlayer,PlaybackSettings};
#[derive(Component)] pub(super) struct PreviewVoice;
#[derive(Resource,Default)] pub(super) struct PreviewRequest(pub Option<Option<(PathBuf,f32)>>,pub Option<(PathBuf,String)>,pub forge_engine::WavEffects,pub Option<f32>);
pub(super) fn process(mut request:ResMut<PreviewRequest>,mut commands:Commands,mut assets:ResMut<Assets<AudioSource>>,mut voices:Query<(Entity,Option<&audio_dsp::DspControl>,Option<&mut bevy::audio::AudioSink>),With<PreviewVoice>>,mut dsp_assets:ResMut<Assets<audio_dsp::DspAudio>>,mut editor:ResMut<Editor>){
 use bevy::audio::AudioSinkPlayback;
 if let Some((root,rel))=request.1.take(){for (entity,_,_) in &mut voices{commands.entity(entity).despawn();}request.0=None;match audio_dsp::DspAudio::load(&root,&rel,request.2,8*1024*1024,true){Ok((source,_))=>{let c=source.control();let gain=request.3.unwrap_or(0.5);commands.spawn((PreviewVoice,AudioPlayer(dsp_assets.add(source)),c,PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(gain))));editor.status="Streaming DSP preview started".into();},Err(err)=>editor.status=err}return;}
 for (_,control,sink) in &mut voices{if let Some(c)=control{c.set(request.2);if let (Some(v),Some(mut sink))=(request.3,sink){sink.set_volume(bevy::audio::Volume::Linear(v));}}}
 let Some(request)=request.0.take()else{return;};for (entity,_,_) in &mut voices{commands.entity(entity).despawn();}
 let Some((path,volume))=request else{return;};
 let result=(||->Result<Vec<u8>,String>{let m=std::fs::metadata(&path).map_err(|e|e.to_string())?;if m.len()>32*1024*1024{return Err("Preview limit: 32 MiB".into());}std::fs::read(path).map_err(|e|e.to_string())})();
 match result{Ok(bytes)=>{let handle=assets.add(AudioSource{bytes:bytes.into()});commands.spawn((PreviewVoice,AudioPlayer::new(handle),PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume.clamp(0.,1.)))));editor.status="Audio preview requested".into();},Err(err)=>editor.status=err}
}
pub(super) fn effects_menu(ui:&mut egui::Ui,e:&mut Editor){ui.collapsing("Offline WAV effects",|ui|{
 let key=egui::Id::new("forge_wav_fx_0720");let mut fx=ui.ctx().data(|d|d.get_temp::<forge_engine::WavEffects>(key)).unwrap_or_default();ui.add(egui::Slider::new(&mut fx.low_db,-12. ..=12.).text("Low dB"));ui.add(egui::Slider::new(&mut fx.high_db,-12. ..=12.).text("High dB"));ui.add(egui::Slider::new(&mut fx.room,0. ..=1.).text("Room reflections"));ui.ctx().data_mut(|d|d.insert_temp(key,fx));ui.label("PCM16 WAV only; creates a new asset, original unchanged.");
 if ui.add_enabled(e.play.is_none(),egui::Button::new("Render WAV effects…")).clicked(){if let Some(root)=e.path.as_ref().and_then(|p|p.parent()){if let Some(file)=rfd::FileDialog::new().add_filter("PCM16 WAV",&["wav"]).pick_file(){match forge_engine::render_wav_effects(&file,root,fx){Ok(rel)=>{if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){o.pro.sound.get_or_insert_with(Default::default).path=rel.clone();}e.status=format!("Rendered {rel}");},Err(err)=>e.status=err}}}else{e.status="Save project first".into();}}
 });}

pub(super) fn live_menu(ui:&mut egui::Ui,e:&mut Editor,request:&mut PreviewRequest){ui.collapsing("Live DSP preview / Аудіоефекти",|ui|{
 ui.small("Preview selected scene PCM16 WAV; loop until Stop preview. Changes are audible while playing.");
 ui.add(egui::Slider::new(&mut request.2.low_db,-12. ..=12.).text("Low dB"));ui.add(egui::Slider::new(&mut request.2.high_db,-12. ..=12.).text("High dB"));ui.add(egui::Slider::new(&mut request.2.room,0. ..=1.).text("Room"));ui.add(egui::Slider::new(&mut request.2.reverb,0. ..=1.).text("Reverb"));let mut gain=request.3.unwrap_or(0.5);ui.add(egui::Slider::new(&mut gain,0. ..=1.).text("Preview gain"));request.3=Some(gain);
 ui.horizontal_wrapped(|ui|{for (name,fx) in forge_engine::DSP_PRESETS{if ui.button(name).clicked(){request.2=fx;}}});
 let sound=e.project.objects.iter().find(|o|Some(o.id)==e.selected).and_then(|o|o.pro.sound.as_ref());let root=e.path.as_ref().and_then(|p|p.parent());
 if ui.add_enabled(sound.is_some()&&root.is_some(),egui::Button::new("Preview selected source (local effects)")).clicked(){if let (Some(s),Some(root))=(sound,root){request.2=s.effects.unwrap_or_default();request.1=Some((root.to_path_buf(),s.path.clone()));}}
 if ui.button("Stop preview").clicked(){request.1=None;request.0=Some(None);}
 if ui.add_enabled(e.play.is_none()&&sound.is_some(),egui::Button::new("Apply effects to selected sound")).clicked(){if let Some(s)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected).and_then(|o|o.pro.sound.as_mut()){s.effects=Some(request.2);}}
 });}
