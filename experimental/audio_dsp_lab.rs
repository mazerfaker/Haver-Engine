//! A standalone audible live DSP test. No integration into normal scene audio yet.
use bevy::{audio::{AddAudioSource,Source},prelude::*,reflect::TypePath};
use forge_engine::{StreamingEffects,WavEffects};
use std::{sync::{Arc,atomic::{AtomicU32,Ordering}},time::Duration};
const RATE:u32=44100;
#[derive(Default)]struct Settings{mode:AtomicU32}
#[derive(Resource)]struct Control(Arc<Settings>);
#[derive(Asset,TypePath)]struct TestAudio{settings:Arc<Settings>}
struct Decoder{settings:Arc<Settings>,dsp:StreamingEffects,sample:u64,mode:u32}
fn params(mode:u32)->WavEffects{match mode{1=>WavEffects{low_db:9.,high_db:-9.,room:0.},2=>WavEffects{low_db:-9.,high_db:9.,room:0.},3=>WavEffects{low_db:0.,high_db:0.,room:1.},_=>WavEffects::default()}}
impl Iterator for Decoder{type Item=f32;fn next(&mut self)->Option<f32>{if self.sample%256==0{let mode=self.settings.mode.load(Ordering::Relaxed);if self.mode!=mode{let _=self.dsp.set_effects(params(mode));self.mode=mode;}}let t=self.sample%u64::from(RATE);let phase=t as f32/RATE as f32;let envelope=(1.-phase/0.12).max(0.);let a=std::f32::consts::TAU*phase;let mut out=[0.12*envelope*((a*160.).sin()+(a*3200.).sin())];self.dsp.process(&mut out);self.sample=self.sample.wrapping_add(1);Some(out[0])}}
impl Source for Decoder{fn current_frame_len(&self)->Option<usize>{None}fn channels(&self)->u16{1}fn sample_rate(&self)->u32{RATE}fn total_duration(&self)->Option<Duration>{None}}
impl Decodable for TestAudio{type DecoderItem=f32;type Decoder=Decoder;fn decoder(&self)->Decoder{Decoder{settings:self.settings.clone(),dsp:StreamingEffects::new(RATE,1,params(0)).expect("valid static DSP configuration"),sample:0,mode:0}}}
fn setup(mut commands:Commands,mut assets:ResMut<Assets<TestAudio>>){let settings=Arc::new(Settings::default());let handle=assets.add(TestAudio{settings:settings.clone()});commands.insert_resource(Control(settings));commands.spawn(Camera2d);commands.spawn(AudioPlayer(handle));println!("Live audio DSP lab: 1 dry, 2 low boost, 3 high boost, 4 room/feedback tail. Esc exits.");}
fn input(keys:Res<ButtonInput<KeyCode>>,control:Res<Control>,mut exit:EventWriter<AppExit>){for (key,mode,label) in [(KeyCode::Digit1,0,"Dry"),(KeyCode::Digit2,1,"Low boost"),(KeyCode::Digit3,2,"High boost"),(KeyCode::Digit4,3,"Room reflections + feedback")]{if keys.just_pressed(key){control.0.mode.store(mode,Ordering::Relaxed);println!("DSP: {label}");}}if keys.just_pressed(KeyCode::Escape){exit.write(AppExit::Success);}}
fn main(){App::new().add_plugins(DefaultPlugins).add_audio_source::<TestAudio>().add_systems(Startup,setup).add_systems(Update,input).run();}
