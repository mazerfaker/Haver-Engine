//! Optional Float32 PCM source with bounded loading and sample-by-sample DSP. Normal audio is the fallback.
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
 fn new(tracks:&[(Arc<[f32]>,f32)])->Self{Self(Arc::from(tracks.iter().map(|(_,g)|AtomicU32::new(g.to_bits())).collect::<Vec<_>>()))}
 pub fn set(&self,index:usize,gain:f32){if let Some(slot)=self.0.get(index){let gain=if gain.is_finite(){gain.clamp(0.,1.)}else{0.};slot.store(gain.to_bits(),Ordering::Relaxed);}}
 fn get(&self,index:usize)->f32{self.0.get(index).map(|g|f32::from_bits(g.load(Ordering::Relaxed))).unwrap_or(0.)}
}
#[derive(Asset,TypePath)]
pub struct DspAudio { pcm:Arc<[f32]>,tracks:Vec<(Arc<[f32]>,f32)>,rate:u32,channels:u16,control:DspControl,looping:bool,gains:TrackGains,cascade:Option<CascadeControl> }
fn parse_format(b:&[u8])->Result<(u32,u16,std::ops::Range<usize>,u16,u16),String>{
 if b.len()<12||&b[..4]!=b"RIFF"||&b[8..12]!=b"WAVE"{return Err("Expected RIFF WAV".into());}
 let mut at=12;let mut fmt=None;let mut data=None;
 while at+8<=b.len(){let n=u32::from_le_bytes(b[at+4..at+8].try_into().unwrap()) as usize;let start=at+8;let end=start.checked_add(n).ok_or("WAV length overflow")?;if end>b.len(){return Err("Truncated WAV".into());}match &b[at..at+4]{b"fmt " if n>=16=>fmt=Some(&b[start..end]),b"data"=>data=Some(start..end),_=>{}}at=end+(n&1);}
 let fmt=fmt.ok_or("Missing format")?;let data=data.ok_or("Missing PCM data")?;let u16_at=|i|u16::from_le_bytes([fmt[i],fmt[i+1]]);let channels=u16_at(2);let rate=u32::from_le_bytes(fmt[4..8].try_into().unwrap());
 let tag=u16_at(0);let bits=u16_at(14);let width=bits/8;
 if !matches!((tag,bits),(1,8|16|24|32)|(3,32))||!(1..=2).contains(&channels)||u16_at(12)!=width*channels||!(8000..=96000).contains(&rate)||data.is_empty()||data.len()%(width as usize*channels as usize)!=0{return Err("Expected nonempty mono/stereo PCM8/16/24/32 or float32 WAV, 8–96 kHz".into());}Ok((rate,channels,data,tag,bits))
}
#[cfg(test)]fn parse(b:&[u8])->Result<(u32,u16,std::ops::Range<usize>),String>{let(r,c,d,tag,bits)=parse_format(b)?;if (tag,bits)!=(1,16){return Err("PCM16 required".into());}Ok((r,c,d))}
/// Decode supported WAV encodings directly into finite Float32 samples, with a byte budget.
fn normalize_pcm(b:&[u8],cap:usize)->Result<(u32,u16,Arc<[f32]>),String>{
 let(rate,channels,range,tag,bits)=parse_format(b)?;let data=&b[range];let width=(bits/8) as usize;let count=data.len()/width;
 if count.checked_mul(4).ok_or("PCM size overflow")?>cap{return Err("Normalized PCM exceeds budget".into());}
 let mut out=Vec::with_capacity(count);for v in data.chunks_exact(width){let f=match(tag,bits){(1,8)=>(f32::from(v[0])-128.)/128.,(1,16)=>i16::from_le_bytes(v.try_into().unwrap()) as f32/32768.,(1,24)=>i32::from_le_bytes([0,v[0],v[1],v[2]]) as f32/2147483648.,(1,32)=>i32::from_le_bytes(v.try_into().unwrap()) as f32/2147483648.,(3,32)=>f32::from_le_bytes(v.try_into().unwrap()),_=>return Err("Unsupported PCM".into())};out.push(if f.is_finite(){f.clamp(-1.,1.)}else{0.});}Ok((rate,channels,Arc::from(out)))
}
/// Linear interpolation outside the audio callback. Shared buses choose highest source rate and channel count.
fn convert_format(pcm:Arc<[f32]>,rate:u32,channels:u16,target_rate:u32,target_channels:u16,cap:usize)->Result<Arc<[f32]>,String>{
 if !matches!(channels,1|2)||!matches!(target_channels,1|2)||!(8000..=96000).contains(&rate)||!(8000..=96000).contains(&target_rate)||pcm.is_empty()||pcm.len()%channels as usize!=0{return Err("Invalid PCM shape".into());}
 let frames=pcm.len()/channels as usize;let dest_frames=((frames as u64*target_rate as u64+rate as u64/2)/rate as u64).max(1) as usize;
 let count=dest_frames.checked_mul(target_channels as usize).ok_or("PCM conversion overflow")?;if count.checked_mul(4).ok_or("PCM conversion overflow")?>cap{return Err("Resampled PCM exceeds budget".into());}if rate==target_rate&&channels==target_channels{return Ok(pcm);}
 let mut out=Vec::with_capacity(count);let sample=|frame:usize,ch:usize|->f32{let base=frame*channels as usize;if channels==2&&target_channels==1{(pcm[base]+pcm[base+1])*0.5}else{pcm[base+ch.min(channels as usize-1)]}};
 for i in 0..dest_frames{let position=i as f64*rate as f64/target_rate as f64;let a=(position.floor() as usize).min(frames-1);let b=(a+1).min(frames-1);let frac=(position-position.floor()) as f32;for ch in 0..target_channels as usize{let x=sample(a,ch);out.push(x+(sample(b,ch)-x)*frac);}}Ok(Arc::from(out))
}

impl DspAudio {
 pub fn control(&self)->DspControl{self.control.clone()}
 pub fn gains(&self)->TrackGains{self.gains.clone()}
 pub fn load(root:&Path,rel:&str,fx:WavEffects,budget:usize,looping:bool)->Result<(Self,usize),String>{
  if !forge_engine::valid_asset(rel){return Err("Invalid audio path".into());}let base=root.join("assets").canonicalize().map_err(|e|e.to_string())?;let path=root.join(rel).canonicalize().map_err(|e|e.to_string())?;if !path.starts_with(base){return Err("Audio outside assets".into());}
  let cap=budget.min(8*1024*1024);let mut raw=Vec::new();std::fs::File::open(path).map_err(|e|e.to_string())?.take(cap as u64+1).read_to_end(&mut raw).map_err(|e|e.to_string())?;if raw.len()>cap{return Err("DSP source budget exceeded".into());}
  let (rate,channels,pcm)=normalize_pcm(&raw,cap)?;StreamingEffects::new(rate,channels as usize,fx)?;let size=pcm.len()*4;Ok((Self{pcm,tracks:Vec::new(),rate,channels,control:DspControl::new(fx),looping,gains:TrackGains::new(&[]),cascade:None},size))
 }
}
pub struct Decoder { pcm:Arc<[f32]>,tracks:Vec<(Arc<[f32]>,f32)>,length:usize,rate:u32,channels:u16,looping:bool,at:usize,tail:usize,tail_total:usize,dsp:StreamingEffects,control:DspControl,seen:u64,until_poll:u16,gains:TrackGains,gain_step:f32,cascade:Option<CascadeDecoder> }
impl Iterator for Decoder {type Item=f32;fn next(&mut self)->Option<f32>{if self.until_poll==0{let next=self.control.snapshot();if next!=self.seen{let _=self.dsp.set_effects(DspControl::decode(next));self.seen=next;}self.until_poll=128;}self.until_poll-=1;let mut sample=[0.];if self.at==self.length{if self.looping{self.at=0;}else if self.tail>0{self.tail-=1;if let Some(c)=&mut self.cascade{sample[0]=c.sample(&self.tracks,self.at,false,true,self.gain_step);}else{self.dsp.process(&mut sample);}return Some(sample[0]);}else{return None;}}sample[0]=if self.tracks.is_empty(){self.pcm[self.at]}else{if self.at%self.channels as usize==0{for (i,(_,gain)) in self.tracks.iter_mut().enumerate(){let target=self.gains.get(i);*gain+=(target-*gain)*self.gain_step;}}if let Some(c)=&mut self.cascade{c.sample(&self.tracks,self.at,self.looping,false,self.gain_step)}else{sum_sample(&self.tracks,self.at,self.looping)}};self.at+=1;if self.cascade.is_none(){self.dsp.process(&mut sample);}Some(sample[0])}}
impl Source for Decoder {fn current_frame_len(&self)->Option<usize>{None}fn channels(&self)->u16{self.channels}fn sample_rate(&self)->u32{self.rate}fn total_duration(&self)->Option<Duration>{if self.looping{None}else{Some(Duration::from_secs_f64((self.length+self.tail_total) as f64/(f64::from(self.channels)*f64::from(self.rate))))}}}
impl Decodable for DspAudio {type DecoderItem=f32;type Decoder=Decoder;fn decoder(&self)->Decoder{let seen=self.control.snapshot();let tail=if !self.looping{self.rate as usize*self.channels as usize*(if self.tracks.is_empty(){1}else{4})/2}else{0};Decoder{pcm:self.pcm.clone(),tracks:self.tracks.clone(),length:self.tracks.iter().map(|(p,_)|p.len()).max().unwrap_or(self.pcm.len()),rate:self.rate,channels:self.channels,looping:self.looping,at:0,tail,tail_total:tail,dsp:StreamingEffects::new(self.rate,self.channels as usize,DspControl::decode(seen)).expect("validated DSP parameters"),control:self.control.clone(),seen,until_poll:0,gains:self.gains.clone(),gain_step:1.-(-1./(self.rate as f32*0.02)).exp(),cascade:self.cascade.as_ref().map(|c|CascadeDecoder::new(c,self.rate,self.channels))}}}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_invalid(){assert!(parse(b"RIFF").is_err());}#[test]fn pcm_bounds(){let mut b=b"RIFF\x26\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x40\x1f\0\0\x80\x3e\0\0\x02\0\x10\0data\x02\0\0\0".to_vec();b.extend_from_slice(&100i16.to_le_bytes());let (rate,channels,r)=parse(&b).unwrap();assert_eq!((rate,channels,r.len()),(8000,1,2));b.pop();assert!(parse(&b).is_err());}}

#[cfg(test)]mod live_control_tests{use super::*;#[test]fn packed_roundtrip(){let c=DspControl::new(WavEffects{low_db:-6.,high_db:8.25,room:0.75,reverb:0.});assert_eq!(DspControl::decode(c.snapshot()),WavEffects{low_db:-6.,high_db:8.25,room:0.75,reverb:0.});c.set(WavEffects::default());assert_eq!(DspControl::decode(c.snapshot()),WavEffects::default());}}

/// Mix aligned interleaved PCM streams before the single streaming effect state.
fn sum_sample(tracks:&[(Arc<[f32]>,f32)],at:usize,looping:bool)->f32{
 let mut sum=0.;for (pcm,gain) in tracks{if pcm.is_empty(){continue;}let pos=if looping{at%pcm.len()}else{at};if pos<pcm.len(){sum+=pcm[pos]* *gain;}}sum
}
impl DspAudio {
 pub fn load_summed(root:&Path,sounds:&[&forge_engine::Sound],fx:WavEffects,budget:usize)->Result<(Self,usize),String>{
  if !(2..=16).contains(&sounds.len()){return Err("Shared bus requires 2–16 sources".into());}let looping=sounds[0].looping;
  if sounds.iter().any(|s|s.spatial||s.stereo_pan||s.speed!=1.||s.effects.is_some()||s.looping!=looping||!s.volume.is_finite()||!(0. ..=1.).contains(&s.volume)){return Err("Incompatible shared bus source".into());}
  let mut loaded=Vec::with_capacity(sounds.len());let(mut rate,mut channels)=(8000u32,1u16);let mut used=0usize;
  for s in sounds{let(source,bytes)=Self::load(root,&s.path,fx,budget.saturating_sub(used),looping)?;rate=rate.max(source.rate);channels=channels.max(source.channels);used+=bytes;loaded.push((source.pcm,source.rate,source.channels,s.volume));}
  let mut tracks=Vec::with_capacity(sounds.len());let mut final_bytes=0usize;
  for(pcm,old_rate,old_channels,gain) in loaded{let original_bytes=pcm.len()*4;let same=old_rate==rate&&old_channels==channels;let cap=if same{original_bytes}else{budget.saturating_sub(used)};let pcm=convert_format(pcm,old_rate,old_channels,rate,channels,cap)?;used=used-original_bytes+pcm.len()*4;final_bytes+=pcm.len()*4;tracks.push((pcm,gain));}
  let used=final_bytes;
  // Equal loop lengths avoid restarting shorter tracks at the longest track's boundary.
  if looping&&tracks.iter().any(|(p,_)|p.len()!=tracks[0].0.len()){return Err("Looping shared tracks must have equal lengths".into());}
  let gains=TrackGains::new(&tracks);Ok((Self{pcm:Arc::from(Vec::<f32>::new()),tracks,rate,channels,control:DspControl::new(fx),looping,gains,cascade:None},used))
 }
}
#[cfg(test)]mod summed35{use super::*;#[test]fn aligned_sum_and_padding(){let a:Arc<[f32]>=Arc::from(vec![0.25f32]);let b:Arc<[f32]>=Arc::from(vec![0.5f32]);let tracks=vec![(a,1.),(b,0.5)];assert_eq!(sum_sample(&tracks,0,false),0.5);assert_eq!(sum_sample(&tracks,2,false),0.);assert_eq!(sum_sample(&tracks,2,true),0.5);}}

/// Opt-in bounded tree of real signal stages; child stages feed parent stages, not merged coefficients.
#[derive(Clone)]struct CascadeStage {name:String,parent:Option<usize>,fx:DspControl,gain:Arc<AtomicU32>}
#[derive(Component,Clone)]pub struct CascadeControl{stages:Arc<[CascadeStage]>,routes:Arc<[usize]>}
impl CascadeControl {
 pub fn update(&self,s:&forge_engine::GameSettings){for stage in self.stages.iter(){if let Some(b)=s.audio_buses.get(&stage.name){stage.fx.set(b.effects.unwrap_or_default());stage.gain.store((if b.mute{0.}else{b.gain.clamp(0.,1.)}).to_bits(),Ordering::Relaxed);}else{stage.gain.store(0f32.to_bits(),Ordering::Relaxed);}}}
}
struct CascadeDecoder{control:CascadeControl,effects:Vec<StreamingEffects>,levels:Vec<f32>,scratch:Vec<f32>,channel:usize,channels:usize,until_poll:u16}
impl CascadeDecoder{
 fn new(control:&CascadeControl,rate:u32,channels:u16)->Self{Self{control:control.clone(),effects:control.stages.iter().map(|s|StreamingEffects::new(rate,channels as usize,DspControl::decode(s.fx.snapshot())).expect("validated format")).collect(),levels:control.stages.iter().map(|s|f32::from_bits(s.gain.load(Ordering::Relaxed))).collect(),scratch:vec![0.;control.stages.len()],channel:0,channels:channels as usize,until_poll:0}}
 fn sample(&mut self,tracks:&[(Arc<[f32]>,f32)],at:usize,looping:bool,tail:bool,step:f32)->f32{
  self.scratch.fill(0.);if self.until_poll==0{for (fx,s) in self.effects.iter_mut().zip(self.control.stages.iter()){let _=fx.set_effects(DspControl::decode(s.fx.snapshot()));}self.until_poll=128;}self.until_poll-=1;
  if !tail{for (i,(pcm,gain)) in tracks.iter().enumerate(){let pos=if looping{at%pcm.len()}else{at};if pos<pcm.len(){self.scratch[self.control.routes[i]]+=pcm[pos]* *gain;}}}
  let mut output=0.;for (i,s) in self.control.stages.iter().enumerate(){if self.channel==0{let g=f32::from_bits(s.gain.load(Ordering::Relaxed));self.levels[i]+=(g-self.levels[i])*step;}let mut sample=[self.scratch[i]];self.effects[i].process(&mut sample);let value=sample[0]*self.levels[i];if let Some(parent)=s.parent{self.scratch[parent]+=value;}else{output+=value;}}
  self.channel=(self.channel+1)%self.channels;output.clamp(-1.,1.)
 }
}
impl DspAudio {
 pub fn load_cascade(root:&Path,sounds:&[&forge_engine::Sound],settings:&forge_engine::GameSettings,bus:&str,budget:usize)->Result<(Self,usize,CascadeControl),String>{
  // Paths must terminate at the selected root and all included formats must match.
  let mut depths=std::collections::BTreeMap::<String,usize>::new();for sound in sounds{let mut name=sound.custom_bus.as_deref().ok_or("Missing route")?;let mut path=Vec::new();for _ in 0..=8{path.push(name.to_owned());if name==bus{break;}name=&settings.audio_buses.get(name).ok_or("Invalid bus route")?.parent;}if path.last().map(String::as_str)!=Some(bus){return Err("Route outside cascade".into());}for (depth,name) in path.into_iter().rev().enumerate(){depths.insert(name,depth);}}
  if !(2..=8).contains(&depths.len()){return Err("Cascade requires 2–8 custom stages".into());}let mut names:Vec<_>=depths.into_iter().collect();names.sort_by(|a,b|b.1.cmp(&a.1).then(a.0.cmp(&b.0)));let mut stages=Vec::new();for (name,_) in &names{let b=settings.audio_buses.get(name).ok_or("Missing stage")?;let parent=if name==bus{None}else{Some(names.iter().position(|(n,_)|n==&b.parent).ok_or("Missing parent stage")?)};stages.push(CascadeStage{name:name.clone(),parent,fx:DspControl::new(b.effects.unwrap_or_default()),gain:Arc::new(AtomicU32::new((if b.mute{0.}else{b.gain.clamp(0.,1.)}).to_bits()))});}
  let routes=sounds.iter().map(|s|names.iter().position(|(n,_)|Some(n.as_str())==s.custom_bus.as_deref()).ok_or("Missing route index")).collect::<Result<Vec<_>,_>>()?;let(mut source,bytes)=Self::load_summed(root,sounds,WavEffects::default(),budget)?;
  let control=CascadeControl{stages:Arc::from(stages),routes:Arc::from(routes)};source.cascade=Some(control.clone());Ok((source,bytes,control))
 }
}


#[cfg(test)]mod cascade_signal37{use super::*;#[test]fn child_then_parent_gain(){let stage=|name:&str,parent,gain:f32|CascadeStage{name:name.into(),parent,fx:DspControl::new(WavEffects::default()),gain:Arc::new(AtomicU32::new(gain.to_bits()))};let control=CascadeControl{stages:Arc::from(vec![stage("child",Some(1),0.5),stage("root",None,0.5)]),routes:Arc::from(vec![0,1])};let mut decoder=CascadeDecoder::new(&control,8000,1);let pcm:Arc<[f32]>=Arc::from(vec![0.5f32]);let tracks=vec![(pcm.clone(),1.),(pcm,1.)];assert!((decoder.sample(&tracks,0,false,false,1.)-0.375).abs()<0.00001);}}

#[cfg(test)]mod formats38{use super::*;fn wav(tag:u16,bits:u16,data:&[u8])->Vec<u8>{let mut b=b"RIFF".to_vec();b.extend_from_slice(&(36+data.len() as u32+(data.len() as u32&1)).to_le_bytes());b.extend_from_slice(b"WAVEfmt ");b.extend_from_slice(&16u32.to_le_bytes());b.extend_from_slice(&tag.to_le_bytes());b.extend_from_slice(&1u16.to_le_bytes());b.extend_from_slice(&8000u32.to_le_bytes());b.extend_from_slice(&(8000*u32::from(bits/8)).to_le_bytes());b.extend_from_slice(&(bits/8).to_le_bytes());b.extend_from_slice(&bits.to_le_bytes());b.extend_from_slice(b"data");b.extend_from_slice(&(data.len() as u32).to_le_bytes());b.extend_from_slice(data);if data.len()%2!=0{b.push(0);}b}#[test]fn mixed_formats(){for (tag,bits,data) in [(1,8,vec![192]),(1,24,vec![0,0,64]),(1,32,vec![0,0,0,64]),(3,32,0.5f32.to_le_bytes().to_vec())]{let (_,_,p)=normalize_pcm(&wav(tag,bits,&data),32).unwrap();assert_eq!(&*p,&[0.5f32]);}assert!(normalize_pcm(&wav(1,8,&[1,2]),3).is_err());}}

#[cfg(test)]mod resample39{use super::*;#[test]fn mono_upmix_and_rate(){let p=convert_format(Arc::from(vec![0.,1.]),8000,1,16000,2,32).unwrap();assert_eq!(&*p,&[0.,0.,0.5,0.5,1.,1.,1.,1.]);}#[test]fn downmix_and_budget(){let p=Arc::from(vec![0.25f32,0.75,0.,1.]);assert_eq!(&*convert_format(p,16000,2,8000,1,4).unwrap(),&[0.5]);assert!(convert_format(Arc::from(vec![0.5f32]),8000,1,96000,2,4).is_err());}#[test]fn float_precision(){let x=0.1234567f32;let p=convert_format(Arc::from(vec![x]),48000,1,48000,1,4).unwrap();assert_eq!(p[0],x);}}
