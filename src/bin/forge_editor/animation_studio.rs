use super::*;
use forge_engine::{ClipCondition,ClipTransition,clip_frame_count,frame_start,set_frame_duration};
pub(super) fn draw(ui:&mut egui::Ui,o:&mut Object,time:&mut f32){
 graph(ui,o,time);
 regions(ui,o,time);
 event_track(ui,o,time);
 markers(ui,o,time);
 ui.collapsing("Animation FSM / Переходи кліпів",|ui|{
  ui.label("Explicit transitions, first matching rule wins. * = any clip. When rules exist, legacy auto-state is not used.");
  let names:Vec<String>=o.clips.keys().cloned().collect();
  if ui.button("+ Transition").clicked(){o.motion.transitions.push(ClipTransition{from:o.clip_name.clone(),to:names.first().cloned().unwrap_or_default(),..Default::default()});}
  let mut remove=None;let mut swap=None;
  for (i,r) in o.motion.transitions.iter_mut().enumerate(){ui.push_id(i,|ui|{ui.horizontal_wrapped(|ui|{
   egui::ComboBox::from_id_salt("from").selected_text(&r.from).show_ui(ui,|ui|{ui.selectable_value(&mut r.from,"*".into(),"Any");for n in &names{ui.selectable_value(&mut r.from,n.clone(),n);}});ui.label("→");
   egui::ComboBox::from_id_salt("to").selected_text(&r.to).show_ui(ui,|ui|{for n in &names{ui.selectable_value(&mut r.to,n.clone(),n);}});
   egui::ComboBox::from_id_salt("condition").selected_text(format!("{:?}",r.condition)).show_ui(ui,|ui|{for c in [ClipCondition::Idle,ClipCondition::Moving,ClipCondition::JumpInput,ClipCondition::Finished,ClipCondition::VariablePositive,ClipCondition::VariableNonPositive]{ui.selectable_value(&mut r.condition,c,format!("{c:?}"));}});
   if ui.small_button("↑").clicked()&&i>0{swap=Some(i);}if ui.small_button("×").clicked(){remove=Some(i);}
  });if matches!(r.condition,ClipCondition::VariablePositive|ClipCondition::VariableNonPositive){ui.text_edit_singleline(&mut r.variable);}if !names.contains(&r.to){ui.colored_label(egui::Color32::LIGHT_RED,"Missing target clip");}});}
  if let Some(i)=swap{o.motion.transitions.swap(i,i-1);}if let Some(i)=remove{o.motion.transitions.remove(i);}
 });
 ui.collapsing("Frame timing / Тривалість кадрів",|ui|{let n=clip_frame_count(&o.animation);let start=o.animation.start.min(o.animation.total_frames()-1);let i=o.animation.frame(*time).saturating_sub(start);ui.label(format!("Frame {} / {} · {:.3} s total",i+1,n,frame_start(&o.animation,n)));let mut ms=((frame_start(&o.animation,i+1)-frame_start(&o.animation,i))*1000.).round().max(1.) as u32;if ui.add(egui::DragValue::new(&mut ms).range(1..=60000).suffix(" ms")).changed(){set_frame_duration(&mut o.animation,i,ms);*time=frame_start(&o.animation,i);}if ui.button("Reset to uniform FPS").clicked(){o.animation.durations.clear();}ui.label("Store clip to persist timing in the named clip.");});
}

/// Select pivot in the current sprite frame, in the same normalized convention as sprite_center.
pub(super) fn pivot(ui:&mut egui::Ui,o:&mut Object,tex:&egui::TextureHandle,time:f32){
 ui.collapsing("Sprite frame pivot / Опорна точка",|ui|{
 let uv=o.animation.uv(time);let ratio=(o.size[0]/o.size[1].max(1.)).clamp(0.1,10.);let w=144_f32.min(144.*ratio);let h=w/ratio;
 let response=ui.add(egui::Image::new((tex.id(),egui::vec2(w,h))).uv(egui::Rect::from_min_max(egui::pos2(uv[0][0],uv[0][1]),egui::pos2(uv[2][0],uv[2][1]))).sense(egui::Sense::click_and_drag()));
 let r=response.rect;if (response.clicked()||response.dragged())&&r.width()>0.&&r.height()>0.{if let Some(pos)=response.interact_pointer_pos(){o.motion.pivot=[((pos.x-r.center().x)/r.width()).clamp(-0.5,0.5),((r.center().y-pos.y)/r.height()).clamp(-0.5,0.5)];}}
 let point=r.center()+egui::vec2(o.motion.pivot[0]*r.width(),-o.motion.pivot[1]*r.height());let painter=ui.painter_at(r);let stroke=egui::Stroke::new(1.5_f32,egui::Color32::YELLOW);painter.line_segment([point-egui::vec2(6.,0.),point+egui::vec2(6.,0.)],stroke);painter.line_segment([point-egui::vec2(0.,6.),point+egui::vec2(0.,6.)],stroke);
 if ui.button("Center pivot").clicked(){o.motion.pivot=[0.;2];}ui.label(format!("X {:.2} | Y {:.2}",o.motion.pivot[0],o.motion.pivot[1]));
 });
}

/// Draggable visual nodes, click two nodes to author a transition; rule list edits conditions.
fn graph(ui:&mut egui::Ui,o:&mut Object,time:&mut f32){ui.collapsing("Animation graph / Граф анімацій",|ui|{
 let names:Vec<String>=o.clips.keys().take(64).cloned().collect();if names.is_empty(){ui.label("Store named clips first");return;}
 let key=ui.id().with(("graph_source",o.id));let mut from=ui.ctx().data(|d|d.get_temp::<String>(key));if from.as_ref().is_some_and(|v|!names.contains(v)){from=None;}
 ui.horizontal_wrapped(|ui|{ui.label(if let Some(s)=&from{format!("Connect {s} → click target")}else{"Click a node to start a transition; drag to move".into()});if ui.button("Cancel link").clicked(){from=None;}if ui.button("Reset positions").clicked(){o.motion.graph_positions.clear();}});
 let width=ui.available_width().max(100.);let height=(names.len().div_ceil(2) as f32*68.+40.).clamp(180.,2300.);let (canvas,_)=ui.allocate_exact_size(egui::vec2(width,height),egui::Sense::hover());let painter=ui.painter_at(canvas);painter.rect_filled(canvas,3.,ui.visuals().extreme_bg_color);
 let node_size=egui::vec2((width*0.40).clamp(40.,140.),46.);let max=egui::vec2((width-node_size.x).max(1.),(height-node_size.y).max(1.));let mut rects=std::collections::BTreeMap::new();
 for (i,n) in names.iter().enumerate(){let pos=o.motion.graph_positions.entry(n.clone()).or_insert([if i%2==0{0.08}else{0.92},((i/2) as f32*68.+16.)/max.y]);let p=canvas.min+egui::vec2(pos[0].clamp(0.,1.)*max.x,pos[1].clamp(0.,1.)*max.y);let r=egui::Rect::from_min_size(p,node_size);let resp=ui.interact(r,ui.id().with(("clip_node",n)),egui::Sense::click_and_drag());if resp.dragged(){let d=ui.input(|i|i.pointer.delta());pos[0]=(pos[0]+d.x/max.x).clamp(0.,1.);pos[1]=(pos[1]+d.y/max.y).clamp(0.,1.);}if resp.double_clicked(){if let Some(a)=o.clips.get(n){o.clip_name=n.clone();o.animation=a.clone();*time=0.;}from=None;}else if resp.clicked(){if let Some(s)=from.take(){if &s!=n&&!o.motion.transitions.iter().any(|t|t.from==s&&t.to==*n){o.motion.transitions.push(ClipTransition{from:s,to:n.clone(),condition:ClipCondition::Moving,variable:String::new()});}}else{from=Some(n.clone());}}resp.context_menu(|ui|{if ui.button("Remove outgoing transitions").clicked(){o.motion.transitions.retain(|t|t.from!=*n);ui.close_menu();}if ui.button("Remove incoming transitions").clicked(){o.motion.transitions.retain(|t|t.to!=*n);ui.close_menu();}});rects.insert(n.clone(),r);}
 for rule in &o.motion.transitions{if let (Some(a),Some(b))=(rects.get(&rule.from),rects.get(&rule.to)){let d=b.center()-a.center();if d.length()<1.{continue;}let dir=d.normalized();let end=b.center()-dir*22.;let start=a.center()+dir*22.;let stroke=egui::Stroke::new(1.5,egui::Color32::from_rgb(116,163,205));painter.arrow(start,end-start,stroke);}}
 for (n,r) in rects{let active=n==o.clip_name;let linking=from.as_ref()==Some(&n);let color=if linking{egui::Color32::from_rgb(120,91,40)}else if active{egui::Color32::from_rgb(42,90,135)}else{ui.visuals().widgets.inactive.bg_fill};painter.rect_filled(r,4.,color);painter.rect_stroke(r,4.,egui::Stroke::new(1.,ui.visuals().widgets.active.bg_stroke.color),egui::StrokeKind::Inside);painter.text(r.center(),egui::Align2::CENTER_CENTER,n,egui::FontId::proportional(11.),ui.visuals().text_color());}
 if o.motion.transitions.iter().any(|r|r.from=="*"){ui.label("Wildcard transitions are edited in the rule list.");}ui.ctx().data_mut(|d|{if let Some(s)=from{d.insert_temp(key,s);}else{d.remove::<String>(key);}});
 });}
fn regions(ui:&mut egui::Ui,o:&mut Object,time:&mut f32){ui.collapsing("Irregular atlas / Області атласу",|ui|{
 if o.animation.regions.is_empty(){if ui.button("Convert current grid to editable regions").clicked(){let c=o.animation.columns.max(1);let r=o.animation.rows.max(1);if c.saturating_mul(r)<=16384{o.animation.regions=(0..c*r).map(|i|[(i%c) as f32/c as f32,(i/c) as f32/r as f32,1./c as f32,1./r as f32]).collect();}}}
 if !o.animation.regions.is_empty(){let i=o.animation.frame(*time) as usize;let n=o.animation.regions.len();ui.label(format!("Region {} / {}",i+1,n));if let Some(region)=o.animation.regions.get_mut(i){let mut next=*region;ui.horizontal_wrapped(|ui|{for (j,label) in ["X","Y","W","H"].iter().enumerate(){ui.add(egui::DragValue::new(&mut next[j]).speed(0.001).range(0. ..=1.).prefix(format!("{label}: ")));}});if forge_engine::valid_region(next){*region=next;}else{ui.colored_label(egui::Color32::LIGHT_RED,"Region must remain within atlas");}}
 if ui.button("Append region").clicked()&&n<16384{o.animation.regions.push([0.,0.,1.,1.]);o.animation.frames=o.animation.regions.len() as u32;o.animation.start=0;o.animation.durations.clear();}
 if ui.button("Return to uniform grid").clicked(){o.animation.regions.clear();o.animation.durations.clear();*time=0.;}ui.label("Store clip to persist region edits. Unequal source sizes use the object size.");}
 });}

/// Full atlas preview, drag selected frame to move, bottom-right handle resizes.
pub(super) fn atlas_canvas(ui:&mut egui::Ui,o:&mut Object,tex:&egui::TextureHandle,time:&mut f32,playing:&mut bool){
 if o.animation.regions.is_empty(){return;}
 ui.collapsing("Atlas canvas / Редактор областей",|ui|{
 let [pw,ph]=tex.size();if pw==0||ph==0{return;}
 let pixels=[pw as u32,ph as u32];let n=clip_frame_count(&o.animation);let start=o.animation.start.min(o.animation.total_frames()-1);let mut idx=o.animation.frame(*time).saturating_sub(start).min(n-1);
 ui.horizontal_wrapped(|ui|{if ui.add(egui::DragValue::new(&mut idx).range(0..=n-1).prefix("Frame (0-based) ")).changed(){*playing=false;*time=forge_engine::frame_sample_time(&o.animation,idx);}if ui.button("Snap to pixels").clicked(){*playing=false;if let Some(r)=o.animation.regions.get_mut((start+idx) as usize){*r=forge_engine::snap_region_pixels(*r,pixels);}}});
 let i=(start+idx) as usize;let Some(region)=o.animation.regions.get_mut(i)else{return;};if !forge_engine::valid_region(*region){return;}
 let scale=(ui.available_width().max(1.)/pw as f32).min(340./ph as f32);let size=egui::vec2(pw as f32*scale,ph as f32*scale);
 let (canvas,_)=ui.allocate_exact_size(size,egui::Sense::hover());let painter=ui.painter_at(canvas);painter.image(tex.id(),canvas,egui::Rect::from_min_max(egui::pos2(0.,0.),egui::pos2(1.,1.)),egui::Color32::WHITE);
 let rect=|r:[f32;4]|egui::Rect::from_min_size(canvas.min+egui::vec2(r[0]*size.x,r[1]*size.y),egui::vec2(r[2]*size.x,r[3]*size.y));let r=rect(*region);let handle=egui::Rect::from_center_size(r.max,egui::vec2(12.,12.)).intersect(canvas);
 let key=ui.id().with(("atlas_edit",o.id,i));let body=ui.interact(r,key.with("move"),egui::Sense::drag());let resize=ui.interact(handle,key.with("resize"),egui::Sense::drag());
 if body.dragged()||resize.dragged(){*playing=false;let d=ui.input(|i|i.pointer.delta());*region=forge_engine::drag_region(*region,[d.x/size.x.max(1.),d.y/size.y.max(1.)],pixels,resize.dragged());}
 let r=rect(*region);let color=egui::Color32::YELLOW;painter.rect_stroke(r,0.,egui::Stroke::new(1.5,color),egui::StrokeKind::Inside);painter.rect_filled(egui::Rect::from_center_size(r.max,egui::vec2(8.,8.)).intersect(canvas),0.,color);
 ui.label(format!("X {:.1}  Y {:.1}  W {:.1}  H {:.1} px",region[0]*pw as f32,region[1]*ph as f32,region[2]*pw as f32,region[3]*ph as f32));ui.small("Drag rectangle to move; bottom-right handle to resize. Snap to pixels, then Store clip.");
 });
}

fn markers(ui:&mut egui::Ui,o:&mut Object,time:&mut f32){ui.collapsing("Animation events / Події",|ui|{
 let end=frame_start(&o.animation,clip_frame_count(&o.animation)).max(0.001);let clip=o.clip_name.clone();if ui.button("Add event at playhead").clicked()&&o.motion.events.len()<256{o.motion.events.push(forge_engine::AnimationEvent{clip:clip.clone(),time:(*time).clamp(0.,(end-0.001).max(0.)),name:"event".into()});}
 let mut remove=None;for (i,e) in o.motion.events.iter_mut().enumerate().filter(|(_,e)|e.clip==clip){ui.push_id(i,|ui|{ui.horizontal_wrapped(|ui|{ui.text_edit_singleline(&mut e.name);ui.add(egui::DragValue::new(&mut e.time).speed(0.01).range(0. ..=(end-0.001).max(0.)).suffix(" s"));if ui.small_button("Seek").clicked(){*time=e.time;}if ui.small_button("×").clicked(){remove=Some(i);}});});}if let Some(i)=remove{o.motion.events.remove(i);}ui.small("Emits a runtime event and calls Lua on_animation_event(name) when available.");
 });ui.collapsing("Transform keyframes / Ключові кадри",|ui|{let mut remove=None;if ui.button("Add transform key at playhead").clicked()&&o.motion.keys.len()<1024{o.motion.keys.retain(|k|(k.time-*time).abs()>0.001);o.motion.keys.push(forge_engine::Key{time:(*time).max(0.),position:o.position,rotation:o.rotation,smooth:false,bezier:None});}for (i,k) in o.motion.keys.iter_mut().enumerate(){ui.push_id(i,|ui|{ui.horizontal_wrapped(|ui|{ui.add(egui::DragValue::new(&mut k.time).range(0. ..=3600.).speed(0.01).suffix(" s"));for v in &mut k.position{ui.add(egui::DragValue::new(v));}ui.add(egui::DragValue::new(&mut k.rotation).suffix("°"));ui.checkbox(&mut k.smooth,"Smooth");if ui.small_button("Seek").clicked(){*time=k.time;}if ui.small_button("×").clicked(){remove=Some(i);}});});}if let Some(i)=remove{o.motion.keys.remove(i);}if ui.button("Sort keys by time").clicked(){o.motion.keys.sort_by(|a,b|a.time.total_cmp(&b.time));}});}

pub(super) fn trim_button(ui:&mut egui::Ui,o:&mut Object,root:&std::path::Path,time:&mut f32,playing:&mut bool,status:&mut String){
 if o.animation.regions.is_empty(){return;}
 let regular=ui.button("Trim transparent margins of selected frame").clicked();
 let can_preserve=o.animation.total_frames()==1&&o.clips.len()<=1&&o.body==forge_engine::Body::None;
 let preserve=ui.add_enabled(can_preserve,egui::Button::new("Trim + preserve world pivot (single frame, no physics)")).clicked();if !regular&&!preserve{return;}
 let result=(||->Result<bool,String>{let rel=o.sprite.as_deref().ok_or("No sprite")?;if !forge_engine::valid_asset(rel){return Err("Invalid sprite path".into());}let path=root.join(rel).canonicalize().map_err(|e|e.to_string())?;if !path.starts_with(root.join("assets").canonicalize().map_err(|e|e.to_string())?){return Err("Sprite outside assets".into());}let mut reader=image::ImageReader::open(path).map_err(|e|e.to_string())?;let mut limits=image::Limits::default();limits.max_image_width=Some(8192);limits.max_image_height=Some(8192);limits.max_alloc=Some(128*1024*1024);reader.limits(limits);let img=reader.decode().map_err(|e|e.to_string())?.to_rgba8();let i=o.animation.frame(*time) as usize;let old=*o.animation.regions.get(i).ok_or("No region")?;if let Some(trim)=forge_engine::trim_alpha_region(&img,old,0){if preserve{let (size,pivot)=forge_engine::compensate_trim(old,trim,o.size,o.motion.pivot).ok_or("Cannot preserve pivot")?;o.size=size;o.motion.pivot=pivot;}o.animation.regions[i]=trim;Ok(true)}else{Ok(false)}})();*playing=false;*status=match result{Ok(true)=>"Trimmed region. Source texture unchanged; Store clip to persist.".into(),Ok(false)=>"No opaque pixels in region; unchanged".into(),Err(e)=>e};
}

fn event_track(ui:&mut egui::Ui,o:&mut Object,time:&mut f32){ui.collapsing("Event timeline / Доріжка подій",|ui|{let duration=frame_start(&o.animation,clip_frame_count(&o.animation)).max(0.001);if !duration.is_finite(){return;}let width=ui.available_width().max(100.);let (r,response)=ui.allocate_exact_size(egui::vec2(width,85.),egui::Sense::click());let painter=ui.painter_at(r);painter.rect_filled(r,4.,ui.visuals().extreme_bg_color);let clip=o.clip_name.clone();let start=r.left()+8.;let span=(r.width()-16.).max(1.);for i in 0..=4{let x=start+span*i as f32/4.;painter.line_segment([egui::pos2(x,r.top()+17.),egui::pos2(x,r.bottom()-5.)],egui::Stroke::new(1.,ui.visuals().widgets.inactive.bg_stroke.color));painter.text(egui::pos2(x,r.top()+4.),egui::Align2::CENTER_TOP,format!("{:.2}",duration*i as f32/4.),egui::FontId::proportional(9.),ui.visuals().text_color());}if response.clicked(){if let Some(pos)=response.interact_pointer_pos(){*time=((pos.x-start)/span).clamp(0.,1.)*(duration-0.001).max(0.);}}for (i,e) in o.motion.events.iter_mut().enumerate().filter(|(_,e)|e.clip==clip){let x=start+span*(e.time/duration).clamp(0.,1.);let y=r.top()+28.+(i%3) as f32*17.;let hit=egui::Rect::from_center_size(egui::pos2(x,y),egui::vec2(16.,16.));let marker=ui.interact(hit,ui.id().with(("event_marker23",o.id,i)),egui::Sense::click_and_drag());if marker.dragged(){let dx=ui.input(|v|v.pointer.delta().x);e.time=(e.time+dx/span*duration).clamp(0.,(duration-0.001).max(0.));*time=e.time;}else if marker.clicked(){*time=e.time;}painter.circle_filled(hit.center(),5.,egui::Color32::from_rgb(230,190,90));painter.text(hit.center()+egui::vec2(7.,0.),egui::Align2::LEFT_CENTER,&e.name,egui::FontId::proportional(10.),ui.visuals().text_color());}let x=start+span*(*time/duration).clamp(0.,1.);painter.line_segment([egui::pos2(x,r.top()+15.),egui::pos2(x,r.bottom())],egui::Stroke::new(1.,egui::Color32::LIGHT_RED));ui.small("Click timeline to seek; drag a marker to change event time.");});}
