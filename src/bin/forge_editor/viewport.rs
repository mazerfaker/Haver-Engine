use super::*;
#[derive(Resource)] pub(super) struct ViewTarget(pub Handle<Image>);
#[derive(Component)] pub(super) struct SceneCamera;
#[derive(Component)] pub(super) struct SceneSprite(Uuid);
#[derive(Default)] pub(super) struct TextureCache(HashMap<PathBuf,(std::time::SystemTime,Handle<Image>)>);
pub(super) fn sync_viewport(mut commands:Commands,e:Res<Editor>,target:Res<ViewTarget>,mut images:ResMut<Assets<Image>>,mut camera:Query<(&mut Transform,&mut Projection,&mut Camera),With<SceneCamera>>,mut sprites:Query<(Entity,&SceneSprite,&mut Sprite,&mut Transform),Without<SceneCamera>>,mut cache:Local<TextureCache>){
 let Ok((mut ct,mut projection,mut cam))=camera.single_mut() else{return;}; cam.is_active=e.native;
 if !e.native && !e.workspace.prefs.camera{return;}
 let [w,h]=e.view_size;if let Some(img)=images.get_mut(&target.0){if img.width()!=w||img.height()!=h{img.resize(Extent3d{width:w,height:h,depth_or_array_layers:1});}}
 if let Projection::Orthographic(p)=&mut *projection{p.scale=1./e.zoom;}
 ct.translation.x=-e.pan.x/e.zoom;ct.translation.y=e.pan.y/e.zoom;
 let project=e.play.as_ref().unwrap_or(&e.project);let time=if e.play.is_some(){e.sim.elapsed}else{e.preview};
 let mut ordered=project.objects.iter().filter(|o|!o.camera).collect::<Vec<_>>();ordered.sort_by(|a,b|a.layer.cmp(&b.layer).then_with(||if project.isometric{project_point(b.position,true)[1].total_cmp(&project_point(a.position,true)[1])}else{std::cmp::Ordering::Equal}));
 for (entity,id,_,_) in &mut sprites{if !ordered.iter().any(|o|o.id==id.0){commands.entity(entity).despawn();}}
 for (i,o) in ordered.iter().enumerate(){let pos=forge_engine::sprite_center(o,project.isometric);let transform=Transform::from_xyz(pos[0],pos[1],i as f32*0.01).with_rotation(Quat::from_rotation_z(o.rotation.to_radians()));let mut sprite=Sprite::from_color(Color::srgba_u8(o.color[0],o.color[1],o.color[2],o.color[3]),Vec2::from_array(o.size));
 if let Some(path)=e.path.as_ref().and_then(|p|p.parent()).zip(o.sprite.as_ref()).map(|(root,rel)|root.join(rel)){if let Ok(stamp)=std::fs::metadata(&path).and_then(|m|m.modified()){if cache.0.get(&path).map(|x|x.0)!=Some(stamp){if let Ok(decoded)=image::open(&path){let rgba=decoded.to_rgba8();let img=Image::new_fill(Extent3d{width:rgba.width(),height:rgba.height(),depth_or_array_layers:1},TextureDimension::D2,rgba.as_raw(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());cache.0.insert(path.clone(),(stamp,images.add(img)));}}if let Some((_,handle))=cache.0.get(&path){sprite.image=handle.clone();if let Some(img)=images.get(handle){let uv=o.animation.uv(time);let size=img.size().as_vec2();sprite.rect=Some(Rect::from_corners(Vec2::from_array(uv[0])*size,Vec2::from_array(uv[2])*size));}}}}
 if let Some((_,_,mut s,mut t))=sprites.iter_mut().find(|(_,id,_,_)|id.0==o.id){*s=sprite;*t=transform;}else{commands.spawn((SceneSprite(o.id),sprite,transform));}}
}

#[derive(Resource)] pub(super) struct GamePreviewTarget(pub Handle<Image>);
#[derive(Component)] pub(super) struct GamePreviewCamera;
pub(super) fn sync_game_camera(e:Res<Editor>,mut q:Query<(&mut Camera,&mut Transform),With<GamePreviewCamera>>){if let Ok((mut camera,mut t))=q.single_mut(){let project=e.play.as_ref().unwrap_or(&e.project);let c=project.objects.iter().find(|o|o.camera);camera.is_active=e.workspace.prefs.camera && c.is_some();if let Some(o)=c{let p=project_point(o.position,project.isometric);t.translation.x=p[0];t.translation.y=p[1];t.rotation=Quat::from_rotation_z(o.rotation.to_radians());}}}
