use super::*;
#[derive(Resource)] pub(super) struct ViewTarget(pub Handle<Image>);
#[derive(Component)] pub(super) struct SceneCamera;
#[derive(Component)] pub(super) struct SceneSprite(Uuid);
type Stamp = std::time::SystemTime;
type Decoded = (PathBuf, Stamp, Result<image::RgbaImage,String>);
pub(super) struct TextureCache {
    ready: HashMap<PathBuf,(Stamp,Handle<Image>,u64)>,
    pending: HashMap<PathBuf,Stamp>, failed: HashMap<PathBuf,Stamp>,
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
  Self{ready:HashMap::new(),pending:HashMap::new(),failed:HashMap::new(),tx,rx:std::sync::Mutex::new(rx)}
 }
}
impl TextureCache {
 fn poll(&mut self,images:&mut Assets<Image>){
  let decoded:Vec<_>=self.rx.lock().unwrap().try_iter().take(4).collect();
  for (path,stamp,result) in decoded {
   self.pending.remove(&path);
   match result {Ok(rgba)=>{
    let bytes=u64::from(rgba.width())*u64::from(rgba.height())*4;
    let used:u64=self.ready.iter().filter(|(k,_)|*k!=&path).map(|(_,v)|v.2).sum();
    if bytes>128*1024*1024 || used.saturating_add(bytes)>128*1024*1024 {self.failed.insert(path,stamp);continue;}
    let img=Image::new_fill(Extent3d{width:rgba.width(),height:rgba.height(),depth_or_array_layers:1},TextureDimension::D2,rgba.as_raw(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());
    self.failed.remove(&path);self.ready.insert(path,(stamp,images.add(img),bytes));
   },Err(_)=>{self.failed.insert(path,stamp);}}
  }
 }
 fn get(&mut self,path:&PathBuf)->Option<Handle<Image>> {
  let stamp=std::fs::metadata(path).and_then(|m|m.modified()).ok()?;
  if self.ready.get(path).map(|x|x.0)!=Some(stamp) && !self.pending.contains_key(path) && self.failed.get(path)!=Some(&stamp) {
   if self.tx.try_send((path.clone(),stamp)).is_ok(){self.pending.insert(path.clone(),stamp);}
  }
  self.ready.get(path).map(|x|x.1.clone())
 }
}
pub(super) fn sync_viewport(mut commands:Commands,e:Res<Editor>,target:Res<ViewTarget>,mut images:ResMut<Assets<Image>>,mut camera:Query<(&mut Transform,&mut Projection,&mut Camera),With<SceneCamera>>,mut sprites:Query<(Entity,&SceneSprite,&mut Sprite,&mut Transform),(Without<SceneCamera>,Without<GamePreviewCamera>)>,mut cache:Local<TextureCache>){
 let Ok((mut ct,mut projection,mut cam))=camera.single_mut() else{return;}; cam.is_active=e.native;
 if !e.native && !e.workspace.prefs.camera && !e.master.game{return;}
 let [w,h]=e.view_size;if images.get(&target.0).is_some_and(|img|img.width()!=w||img.height()!=h){if let Some(img)=images.get_mut(&target.0){img.resize(Extent3d{width:w,height:h,depth_or_array_layers:1});}}
 if let Projection::Orthographic(p)=&mut *projection{p.scale=1./e.zoom;}
 ct.translation.x=-e.pan.x/e.zoom;ct.translation.y=e.pan.y/e.zoom;
 cache.poll(&mut images); let project=e.play.as_ref().unwrap_or(&e.project);let time=if e.play.is_some(){e.sim.elapsed}else{e.preview};
 let mut ordered=project.objects.iter().filter(|o|!o.camera).collect::<Vec<_>>();ordered.sort_by(|a,b|a.layer.cmp(&b.layer).then_with(||if project.isometric{project_point(b.position,true)[1].total_cmp(&project_point(a.position,true)[1])}else{std::cmp::Ordering::Equal}));
 let active:std::collections::HashSet<_>=ordered.iter().map(|o|o.id).collect();
 let existing:HashMap<_,_>=sprites.iter().map(|(entity,id,_,_)|(id.0,entity)).collect();
 for (entity,id,_,_) in &mut sprites{if !active.contains(&id.0){commands.entity(entity).despawn();}}
 let paths:std::collections::HashSet<_>=ordered.iter().filter_map(|o|e.path.as_ref().and_then(|p|p.parent()).zip(o.sprite.as_ref()).map(|(root,rel)|root.join(rel))).collect();
 let before=cache.ready.len();cache.ready.retain(|path,_|paths.contains(path));if before!=cache.ready.len(){cache.failed.clear();}cache.failed.retain(|path,_|paths.contains(path));
 for (i,o) in ordered.iter().enumerate(){let pos=forge_engine::sprite_center(o,project.isometric);let transform=Transform::from_xyz(pos[0],pos[1],i as f32*0.01).with_rotation(Quat::from_rotation_z(o.rotation.to_radians()));let c=forge_engine::lit_color(o,project);let mut sprite=Sprite::from_color(Color::srgba_u8(c[0],c[1],c[2],c[3]),Vec2::from_array(o.size));
 if let Some(path)=e.path.as_ref().and_then(|p|p.parent()).zip(o.sprite.as_ref()).map(|(root,rel)|root.join(rel)) {if let Some(handle)=cache.get(&path){sprite.image=handle.clone();if let Some(img)=images.get(&handle){let uv=o.animation.uv(time);let size=img.size().as_vec2();sprite.rect=Some(Rect::from_corners(Vec2::from_array(uv[0])*size,Vec2::from_array(uv[2])*size));}}}
 if let Some(entity)=existing.get(&o.id){if let Ok((_,_,mut s,mut t))=sprites.get_mut(*entity){*s=sprite;*t=transform;}}else{commands.spawn((SceneSprite(o.id),sprite,transform));}}
}

#[derive(Resource)] pub(super) struct GamePreviewTarget(pub Handle<Image>);
#[derive(Component)] pub(super) struct GamePreviewCamera;
pub(super) fn sync_game_camera(e:Res<Editor>,target:Res<GamePreviewTarget>,mut images:ResMut<Assets<Image>>,mut q:Query<(&mut Camera,&mut Transform),With<GamePreviewCamera>>){if let Ok((mut camera,mut t))=q.single_mut(){let project=e.play.as_ref().unwrap_or(&e.project);let c=project.objects.iter().find(|o|o.camera);camera.is_active=(e.workspace.prefs.camera || e.master.game) && c.is_some();if !camera.is_active{return;}let [w,h]=e.master.settings.resolution;if images.get(&target.0).is_some_and(|img|img.width()!=w||img.height()!=h){if let Some(img)=images.get_mut(&target.0){img.resize(Extent3d{width:w,height:h,depth_or_array_layers:1});}}if let Some(o)=c{let p=project_point(o.position,project.isometric);t.translation.x=p[0];t.translation.y=p[1];t.rotation=Quat::from_rotation_z(o.rotation.to_radians());}}}
#[derive(Component)] pub(super) struct PreviewParticle;
pub(super) fn preview_particles(mut commands:Commands,e:Res<Editor>,mut old:Query<(Entity,&mut Sprite,&mut Transform),With<PreviewParticle>>){
 let mut pool=old.iter_mut();let mut count=0;
 if let Some(p)=e.play.as_ref(){for o in &p.objects{for (pos,size,c) in forge_engine::particles(o,e.sim.elapsed){if count>=4096{break;}count+=1;let pos=project_point(pos,p.isometric);let sprite=Sprite::from_color(Color::srgba_u8(c[0],c[1],c[2],c[3]),Vec2::splat(size));let transform=Transform::from_xyz(pos[0],pos[1],100.);if let Some((_,mut s,mut t))=pool.next(){*s=sprite;*t=transform;}else{commands.spawn((PreviewParticle,sprite,transform));}}}}
 for (id,_,_) in pool{commands.entity(id).despawn();}
}
