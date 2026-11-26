//! Optional PCM16 source with bounded loading and sample-by-sample DSP. Normal audio is the fallback.
use bevy::{audio::{Decodable,Source},prelude::*,reflect::TypePath};
use forge_engine::{StreamingEffects,WavEffects};
use std::{path::Path,sync::{Arc,atomic::{AtomicU64,AtomicU32,Ordering}},time::Duration,io::Read};
/// One coherent atomic snapshot; no mutex or allocation in the decoder callback.
#[derive(Component,Clone)]
pub struct DspControl(Arc<AtomicU64>);
impl DspControl {
 fn encode(fx:WavEffects)->u64 {let q=|v:f32,lo:f32,hi:f32| (((if v.is_finite(){v.clamp(lo,hi)}else{0.})-lo)*1000.).round() as u64;q(fx.low_db,-12.,12.)|(q(fx.high_db,-12.,12.)<<16)|(q(fx.room,0.,1.)<<32)|(q(fx.reverb,0.,1.)<<48)}
 fn decode(v:u64)->WavEffects {WavEffects{low_db:(v&65535) as f32/1000.-12.,high_db:((v>>16)&65535) as f32/1000.-12.,room:((v>>32)&65535) as f32/1000.,reverb:((v>>48)&65535) as f32/1000.}}
 pub fn new(fx:WavEffects)->Self{Self(Arc::new(AtomicU64::new(Self::encode(fx))))}
 pub fn set(&self,fx:WavEffects){let packed=Self::encode(fx);if self.0.load(Ordering::Relaxed)!=packed{self.0.store(packed,Ordering::Relaxed);}}
 fn snapshot(&self)->u64{self.0.load(Ordering::Relaxed)}
}
#[derive(Component,Clone)]
pub struct TrackGains(Arc<[AtomicU32]>);
impl TrackGains {
 fn new(tracks:&[(Arc<[u8]>,f32)])->Self{Self(Arc::from(tracks.iter().map(|(_,g)|AtomicU32::new(g.to_bits())).collect::<Vec<_>>()))}
 pub fn set(&self,index:usize,gain:f32){if let Some(slot)=self.0.get(index){let gain=if gain.is_finite(){gain.clamp(0.,1.)}else{0.};slot.store(gain.to_bits(),Ordering::Relaxed);}}
 fn get(&self,index:usize)->f32{self.0.get(index).map(|g|f32::from_bits(g.load(Ordering::Relaxed))).unwrap_or(0.)}
}
#[derive(Asset,TypePath)]
pub struct DspAudio { pcm:Arc<[u8]>,tracks:Vec<(Arc<[u8]>,f32)>,rate:u32,channels:u16,control:DspControl,looping:bool,gains:TrackGains }
fn parse(b:&[u8])->Result<(u32,u16,std::ops::Range<usize>),String>{
 if b.len()<12||&b[..4]!=b"RIFF"||&b[8..12]!=b"WAVE"{return Err("Expected RIFF WAV".into());}
 let mut at=12;let mut fmt=None;let mut data=None;
 while at+8<=b.len(){let n=u32::from_le_bytes(b[at+4..at+8].try_into().unwrap()) as usize;let start=at+8;let end=start.checked_add(n).ok_or("WAV length overflow")?;if end>b.len(){return Err("Truncated WAV".into());}match &b[at..at+4]{b"fmt " if n>=16=>fmt=Some(&b[start..end]),b"data"=>data=Some(start..end),_=>{}}at=end+(n&1);}
 let fmt=fmt.ok_or("Missing format")?;let data=data.ok_or("Missing PCM data")?;let u16_at=|i|u16::from_le_bytes([fmt[i],fmt[i+1]]);let channels=u16_at(2);let rate=u32::from_le_bytes(fmt[4..8].try_into().unwrap());
 if u16_at(0)!=1||!(1..=2).contains(&channels)||u16_at(14)!=16||u16_at(12)!=2*channels||!(8000..=96000).contains(&rate)||data.is_empty()||data.len()%(2*channels as usize)!=0{return Err("Only nonempty mono/stereo PCM16 WAV, 8–96 kHz".into());}Ok((rate,channels,data))
}
impl DspAudio {
 pub fn control(&self)->DspControl{self.control.clone()}
 pub fn gains(&self)->TrackGains{self.gains.clone()}
 pub fn load(root:&Path,rel:&str,fx:WavEffects,budget:usize,looping:bool)->Result<(Self,usize),String>{
  if !forge_engine::valid_asset(rel){return Err("Invalid audio path".into());}let base=root.join("assets").canonicalize().map_err(|e|e.to_string())?;let path=root.join(rel).canonicalize().map_err(|e|e.to_string())?;if !path.starts_with(base){return Err("Audio outside assets".into());}
  let cap=budget.min(8*1024*1024);let mut raw=Vec::new();std::fs::File::open(path).map_err(|e|e.to_string())?.take(cap as u64+1).read_to_end(&mut raw).map_err(|e|e.to_string())?;if raw.len()>cap{return Err("DSP source budget exceeded".into());}
  let (rate,channels,range)=parse(&raw)?;StreamingEffects::new(rate,channels as usize,fx)?;let size=range.len();Ok((Self{pcm:Arc::from(&raw[range]),tracks:Vec::new(),rate,channels,control:DspControl::new(fx),looping,gains:TrackGains::new(&[])},size))
 }
}
pub struct Decoder { pcm:Arc<[u8]>,tracks:Vec<(Arc<[u8]>,f32)>,length:usize,rate:u32,channels:u16,looping:bool,at:usize,tail:usize,tail_total:usize,dsp:StreamingEffects,control:DspControl,seen:u64,until_poll:u16,gains:TrackGains,gain_step:f32 }
impl Iterator for Decoder {type Item=f32;fn next(&mut self)->Option<f32>{if self.until_poll==0{let next=self.control.snapshot();if next!=self.seen{let _=self.dsp.set_effects(DspControl::decode(next));self.seen=next;}self.until_poll=128;}self.until_poll-=1;let mut sample=[0.];if self.at==self.length{if self.looping{self.at=0;}else if self.tail>0{self.tail-=1;self.dsp.process(&mut sample);return Some(sample[0]);}else{return None;}}sample[0]=if self.tracks.is_empty(){i16::from_le_bytes([self.pcm[self.at],self.pcm[self.at+1]]) as f32/32768.}else{if self.at%(2*self.channels as usize)==0{for (i,(_,gain)) in self.tracks.iter_mut().enumerate(){let target=self.gains.get(i);*gain+=(target-*gain)*self.gain_step;}}sum_sample(&self.tracks,self.at,self.looping)};self.at+=2;self.dsp.process(&mut sample);Some(sample[0])}}
impl Source for Decoder {fn current_frame_len(&self)->Option<usize>{None}fn channels(&self)->u16{self.channels}fn sample_rate(&self)->u32{self.rate}fn total_duration(&self)->Option<Duration>{if self.looping{None}else{Some(Duration::from_secs_f64((self.length/2+self.tail_total) as f64/(f64::from(self.channels)*f64::from(self.rate))))}}}
impl Decodable for DspAudio {type DecoderItem=f32;type Decoder=Decoder;fn decoder(&self)->Decoder{let seen=self.control.snapshot();let tail=if !self.looping{self.rate as usize*self.channels as usize*(if self.tracks.is_empty(){1}else{4})/2}else{0};Decoder{pcm:self.pcm.clone(),tracks:self.tracks.clone(),length:self.tracks.iter().map(|(p,_)|p.len()).max().unwrap_or(self.pcm.len()),rate:self.rate,channels:self.channels,looping:self.looping,at:0,tail,tail_total:tail,dsp:StreamingEffects::new(self.rate,self.channels as usize,DspControl::decode(seen)).expect("validated DSP parameters"),control:self.control.clone(),seen,until_poll:0,gains:self.gains.clone(),gain_step:1.-(-1./(self.rate as f32*0.02)).exp()}}}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_invalid(){assert!(parse(b"RIFF").is_err());}#[test]fn pcm_bounds(){let mut b=b"RIFF\x26\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x40\x1f\0\0\x80\x3e\0\0\x02\0\x10\0data\x02\0\0\0".to_vec();b.extend_from_slice(&100i16.to_le_bytes());let (rate,channels,r)=parse(&b).unwrap();assert_eq!((rate,channels,r.len()),(8000,1,2));b.pop();assert!(parse(&b).is_err());}}

#[cfg(test)]mod live_control_tests{use super::*;#[test]fn packed_roundtrip(){let c=DspControl::new(WavEffects{low_db:-6.,high_db:8.25,room:0.75,reverb:0.});assert_eq!(DspControl::decode(c.snapshot()),WavEffects{low_db:-6.,high_db:8.25,room:0.75,reverb:0.});c.set(WavEffects::default());assert_eq!(DspControl::decode(c.snapshot()),WavEffects::default());}}

/// Mix aligned interleaved PCM streams before the single streaming effect state.
fn sum_sample(tracks:&[(Arc<[u8]>,f32)],at:usize,looping:bool)->f32{
 let mut sum=0.;for (pcm,gain) in tracks{if pcm.is_empty(){continue;}let pos=if looping{at%pcm.len()}else{at};if pos+1<pcm.len(){sum+=i16::from_le_bytes([pcm[pos],pcm[pos+1]]) as f32/32768. * *gain;}}sum
}
impl DspAudio {
 pub fn load_summed(root:&Path,sounds:&[&forge_engine::Sound],fx:WavEffects,budget:usize)->Result<(Self,usize),String>{
  if !(2..=16).contains(&sounds.len()){return Err("Shared bus requires 2–16 sources".into());}let looping=sounds[0].looping;
  if sounds.iter().any(|s|s.spatial||s.stereo_pan||s.speed!=1.||s.effects.is_some()||s.looping!=looping||!s.volume.is_finite()||!(0. ..=1.).contains(&s.volume)){return Err("Incompatible shared bus source".into());}
  let mut tracks=Vec::with_capacity(sounds.len());let mut format=None;let mut used=0usize;
  for s in sounds{let (source,bytes)=Self::load(root,&s.path,fx,budget.saturating_sub(used),looping)?;let shape=(source.rate,source.channels);if format.is_some_and(|v|v!=shape){return Err("Shared bus formats differ".into());}format=Some(shape);used+=bytes;tracks.push((source.pcm,s.volume));}
  let(rate,channels)=format.ok_or("No shared sources")?;
  // Equal loop lengths avoid restarting shorter tracks at the longest track's boundary.
  if looping&&tracks.iter().any(|(p,_)|p.len()!=tracks[0].0.len()){return Err("Looping shared tracks must have equal lengths".into());}
  let gains=TrackGains::new(&tracks);Ok((Self{pcm:Arc::from(Vec::<u8>::new()),tracks,rate,channels,control:DspControl::new(fx),looping,gains},used))
 }
}
#[cfg(test)]mod summed35{use super::*;#[test]fn aligned_sum_and_padding(){let a:Arc<[u8]>=Arc::from(8192i16.to_le_bytes().to_vec());let b:Arc<[u8]>=Arc::from(16384i16.to_le_bytes().to_vec());let tracks=vec![(a,1.),(b,0.5)];assert_eq!(sum_sample(&tracks,0,false),0.5);assert_eq!(sum_sample(&tracks,2,false),0.);assert_eq!(sum_sample(&tracks,2,true),0.5);}}
