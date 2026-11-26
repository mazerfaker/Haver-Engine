use super::*;
#[derive(Resource)] pub(super) struct ViewTarget(pub Handle<Image>);
#[derive(Component)] pub(super) struct SceneCamera;
#[derive(Component)] pub(super) struct SceneSprite(Uuid);
use std::sync::atomic::{AtomicU64,Ordering};
static USED:AtomicU64=AtomicU64::new(0);static HITS:AtomicU64=AtomicU64::new(0);static MISSES:AtomicU64=AtomicU64::new(0);static EVICTIONS:AtomicU64=AtomicU64::new(0);
pub(super) fn cache_stats()->(u64,u64,u64,u64){(USED.load(Ordering::Relaxed),HITS.load(Ordering::Relaxed),MISSES.load(Ordering::Relaxed),EVICTIONS.load(Ordering::Relaxed))}
type Stamp = std::time::SystemTime;
type Decoded = (PathBuf, Stamp, Result<image::RgbaImage,String>);
pub(super) struct TextureCache {
    ready: HashMap<PathBuf,(Stamp,Handle<Image>,u64,u64)>,
 clock:u64,budget:u64, pinned:std::collections::HashSet<PathBuf>,
    pending: HashMap<PathBuf,Stamp>, failed: HashMap<PathBuf,Stamp>,
    checks: HashMap<PathBuf,(std::time::Instant,Option<Stamp>)>,
    tx: std::sync::mpsc::SyncSender<(PathBuf,Stamp)>,
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<Decoded>>,
}
impl Default for TextureCache {
 fn default()->Self {
  let (tx,jobs)=std::sync::mpsc::sync_channel::<(PathBuf,Stamp)>(8);
  let (done,rx)=std::sync::mpsc::sync_channel::<Decoded>(8);
  std::thread::spawn(move || while let Ok((path,stamp))=jobs.recv(){
   let result=(|| -> Result<image::RgbaImage,String> {let mut reader=image::ImageReader::open(&path).map_err(|e|e.to_string())?;let mut limits=image::Limits::default();limits.max_image_width=Some(8192);limits.max_image_height=Some(8192);limits.max_alloc=Some(128*1024*1024);reader.limits(limits);reader.decode().map(|img|img.to_rgba8()).map_err(|e|e.to_string())})();
   if done.send((path,stamp,result)).is_err(){break;}
  });
  Self{clock:0,budget:128*1024*1024,pinned:Default::default(),ready:HashMap::new(),pending:HashMap::new(),failed:HashMap::new(),checks:HashMap::new(),tx,rx:std::sync::Mutex::new(rx)}
 }
}
impl TextureCache {
 fn poll(&mut self,images:&mut Assets<Image>){
  let decoded:Vec<_>=self.rx.lock().unwrap().try_iter().take(4).collect();
  for (path,stamp,result) in decoded {
   self.pending.remove(&path);
   match result {Ok(rgba)=>{
    let bytes=u64::from(rgba.width())*u64::from(rgba.height())*4;
    if bytes>self.budget {self.failed.insert(path,stamp);continue;}
    let mut used:u64=self.ready.iter().filter(|(k,_)|*k!=&path).map(|(_,v)|v.2).sum();
    while used.saturating_add(bytes)>self.budget {
     let victim=self.ready.iter().filter(|(k,_)|*k!=&path&&!self.pinned.contains(*k)).min_by_key(|(_,v)|v.3).map(|(k,_)|k.clone());
     let Some(victim)=victim else {break;};
     if let Some(v)=self.ready.remove(&victim){used=used.saturating_sub(v.2);EVICTIONS.fetch_add(1,Ordering::Relaxed);}
    }
    if used.saturating_add(bytes)>self.budget {self.failed.insert(path,stamp);continue;}
    let img=Image::new_fill(Extent3d{width:rgba.width(),height:rgba.height(),depth_or_array_layers:1},TextureDimension::D2,rgba.as_raw(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());
    self.failed.remove(&path);self.ready.insert(path,(stamp,images.add(img),bytes,self.clock));
   },Err(_)=>{self.failed.insert(path,stamp);}}
  }
 }
 fn get(&mut self,path:&PathBuf)->Option<Handle<Image>> {
  self.clock=self.clock.wrapping_add(1);let now=std::time::Instant::now();let check=self.checks.entry(path.clone()).or_insert_with(||(now,std::fs::metadata(path).and_then(|m|m.modified()).ok()));if now.duration_since(check.0)>=std::time::Duration::from_millis(500){*check=(now,std::fs::metadata(path).and_then(|m|m.modified()).ok());}let stamp=check.1?;
  if self.ready.get(path).map(|x|x.0)!=Some(stamp) && !self.pending.contains_key(path) && self.failed.get(path)!=Some(&stamp) {
   if self.tx.try_send((path.clone(),stamp)).is_ok(){self.pending.insert(path.clone(),stamp);}
  }
  if let Some(x)=self.ready.get_mut(path){x.3=self.clock;HITS.fetch_add(1,Ordering::Relaxed);Some(x.1.clone())}else{MISSES.fetch_add(1,Ordering::Relaxed);None}
 }
}
pub(super) fn sync_viewport(mut commands:Commands,e:Res<Editor>,target:Res<ViewTarget>,mut images:ResMut<Assets<Image>>,mut camera:Query<(&mut Transform,&mut Projection,&mut Camera),With<SceneCamera>>,mut sprites:Query<(Entity,&SceneSprite,&mut Sprite,&mut Transform),(Without<SceneCamera>,Without<GamePreviewCamera>)>,mut cache:Local<TextureCache>){
 let Ok((mut ct,mut projection,mut cam))=camera.single_mut() else{return;}; cam.is_active=e.native;
 if !e.native && !e.workspace.prefs.camera && !e.master.game{return;}
 let [w,h]=e.view_size;if images.get(&target.0).is_some_and(|img|img.width()!=w||img.height()!=h){if let Some(img)=images.get_mut(&target.0){img.resize(Extent3d{width:w,height:h,depth_or_array_layers:1});}}
 if let Projection::Orthographic(p)=&mut *projection{p.scale=1./e.zoom;}
 ct.translation.x=-e.pan.x/e.zoom;ct.translation.y=e.pan.y/e.zoom;
 let budget=u64::from(e.master.settings.cache_mib.clamp(32,1024))*1048576;if budget!=cache.budget{cache.budget=budget;cache.failed.clear();while cache.ready.values().map(|v|v.2).sum::<u64>()>budget{let key=cache.ready.iter().min_by_key(|(_,v)|v.3).map(|(k,_)|k.clone());if let Some(k)=key{cache.ready.remove(&k);EVICTIONS.fetch_add(1,Ordering::Relaxed);}else{break;}}}let project=e.play.as_ref().unwrap_or(&e.project);let time=if e.play.is_some(){e.sim.elapsed}else{e.preview};
 let mut ordered=project.objects.iter().filter(|o|!o.camera).collect::<Vec<_>>();ordered.sort_by(|a,b|a.layer.cmp(&b.layer).then_with(||if project.isometric{project_point(b.position,true)[1].total_cmp(&project_point(a.position,true)[1])}else{std::cmp::Ordering::Equal}));
 let active:std::collections::HashSet<_>=ordered.iter().map(|o|o.id).collect();
 let existing:HashMap<_,_>=sprites.iter().map(|(entity,id,_,_)|(id.0,entity)).collect();
 for (entity,id,_,_) in &mut sprites{if !active.contains(&id.0){commands.entity(entity).despawn();}}
 let paths:std::collections::HashSet<_>=ordered.iter().filter_map(|o|e.path.as_ref().and_then(|p|p.parent()).zip(o.sprite.as_ref()).map(|(root,rel)|root.join(rel))).collect();
 let new_pins:std::collections::HashSet<_>=ordered.iter().filter(|o|e.workspace.prefs.camera || e.master.game || forge_engine::spatial::visible(forge_engine::sprite_center(o,project.isometric),o.size,o.rotation,[ct.translation.x,ct.translation.y],[w as f32/(2.*e.zoom),h as f32/(2.*e.zoom)])).filter_map(|o|e.path.as_ref().and_then(|p|p.parent()).zip(o.sprite.as_ref()).map(|(root,rel)|root.join(rel))).collect();
 if new_pins!=cache.pinned {cache.failed.clear();cache.pinned=new_pins;}
 let before=cache.ready.len();cache.ready.retain(|path,_|paths.contains(path));if before!=cache.ready.len(){EVICTIONS.fetch_add((before-cache.ready.len()) as u64,Ordering::Relaxed);cache.failed.clear();}cache.failed.retain(|path,_|paths.contains(path));cache.checks.retain(|path,_|paths.contains(path));
 cache.poll(&mut images);
 USED.store(cache.ready.values().map(|v|v.2).sum(),Ordering::Relaxed);for (i,o) in ordered.iter().enumerate(){let pos=forge_engine::sprite_center(o,project.isometric);let (frame_center,frame_size)=forge_engine::sprite_frame_geometry(o,project.isometric,forge_engine::animation_time(o,time));let transform=Transform::from_xyz(frame_center[0],frame_center[1],i as f32*0.01).with_rotation(Quat::from_rotation_z(o.rotation.to_radians()));let c=forge_engine::lit_color(o,project);let mut sprite=Sprite::from_color(Color::srgba_u8(c[0],c[1],c[2],c[3]),Vec2::from_array(frame_size));
 let visible=e.workspace.prefs.camera || e.master.game || forge_engine::spatial::visible(pos,o.size,o.rotation,[ct.translation.x,ct.translation.y],[w as f32/(2.*e.zoom),h as f32/(2.*e.zoom)]);if visible {if let Some(path)=e.path.as_ref().and_then(|p|p.parent()).zip(o.sprite.as_ref()).map(|(root,rel)|root.join(rel)) {if let Some(handle)=cache.get(&path){sprite.image=handle.clone();if let Some(img)=images.get(&handle){let uv=o.animation.uv(forge_engine::animation_time(o,time));let size=img.size().as_vec2();sprite.rect=Some(Rect::from_corners(Vec2::from_array(uv[0])*size,Vec2::from_array(uv[2])*size));}}}}
 if let Some(entity)=existing.get(&o.id){if let Ok((_,_,mut s,mut t))=sprites.get_mut(*entity){*s=sprite;*t=transform;}}else{commands.spawn((SceneSprite(o.id),sprite,transform));}}
}

#[derive(Resource)] pub(super) struct GamePreviewTarget(pub Handle<Image>);
#[derive(Component)] pub(super) struct GamePreviewCamera;
pub(super) fn sync_game_camera(e:Res<Editor>,target:Res<GamePreviewTarget>,mut images:ResMut<Assets<Image>>,mut q:Query<(&mut Camera,&mut Transform),With<GamePreviewCamera>>){if let Ok((mut camera,mut t))=q.single_mut(){let project=e.play.as_ref().unwrap_or(&e.project);let c=project.objects.iter().find(|o|o.camera);camera.is_active=(e.workspace.prefs.camera || e.master.game) && c.is_some();if !camera.is_active{return;}let [w,h]=e.master.settings.resolution;if images.get(&target.0).is_some_and(|img|img.width()!=w||img.height()!=h){if let Some(img)=images.get_mut(&target.0){img.resize(Extent3d{width:w,height:h,depth_or_array_layers:1});}}if let Some(o)=c{let p=project_point(o.position,project.isometric);t.translation.x=p[0];t.translation.y=p[1];t.rotation=Quat::from_rotation_z(o.rotation.to_radians());}}}
#[derive(Component)] pub(super) struct PreviewParticle;
pub(super) fn preview_particles(mut commands:Commands,e:Res<Editor>,mut old:Query<(Entity,&mut Sprite,&mut Transform),With<PreviewParticle>>){
 let mut pool=old.iter_mut();let mut count=0;
 if let Some(p)=e.play.as_ref(){let gpu=forge_engine::gpu_particles::selected(p);for o in &p.objects{if gpu.contains(&o.id){continue;}for (pos,size,c) in forge_engine::particles(o,e.sim.elapsed){if count>=4096{break;}count+=1;let pos=project_point(pos,p.isometric);let sprite=Sprite::from_color(Color::srgba_u8(c[0],c[1],c[2],c[3]),Vec2::splat(size));let transform=Transform::from_xyz(pos[0],pos[1],100.);if let Some((_,mut s,mut t))=pool.next(){*s=sprite;*t=transform;}else{commands.spawn((PreviewParticle,sprite,transform));}}}}
 for (id,_,_) in pool{commands.entity(id).despawn();}
}

pub(super) fn gpu_scene_sync(e:Res<Editor>,mut draw:forge_engine::gpu_scene::GpuDraw,mut sprites:Query<(Entity,&SceneSprite,&Sprite,&Transform,&mut Visibility)>){
 let p=e.play.as_ref().unwrap_or(&e.project);if (!p.settings.gpu_lighting||p.isometric)&&draw.idle(){return;}let elapsed=if e.play.is_some(){e.sim.elapsed}else{e.preview};let objects:HashMap<_,_>=p.objects.iter().map(|o|(o.id,o)).collect();
 draw.sync(p,e.path.as_ref().and_then(|p|p.parent()),elapsed,sprites.iter_mut().filter_map(|(entity,id,s,t,v)|objects.get(&id.0).map(|o|(entity,*o,s,t,v))));
}

pub(super) fn gpu_particles_sync(e:Res<Editor>,time:Res<Time>,mut draw:forge_engine::gpu_particles::ParticleDraw){let p=e.play.as_ref().unwrap_or(&e.project);let dt=if e.play.is_some()&&e.master.paused{0.}else{time.delta_secs()};draw.sync(p,e.path.as_ref().and_then(|p|p.parent()),dt);}
