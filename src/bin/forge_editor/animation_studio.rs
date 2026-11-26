use super::*;
use forge_engine::{ClipCondition,ClipTransition,clip_frame_count,frame_start,set_frame_duration};
pub(super) fn draw(ui:&mut egui::Ui,o:&mut Object,time:&mut f32){
 ui.collapsing("Animation FSM / Переходи кліпів",|ui|{
  ui.label("Explicit transitions, first matching rule wins. * = any clip. When rules exist, legacy auto-state is not used.");
  let names:Vec<String>=o.clips.keys().cloned().collect();
  if ui.button("+ Transition").clicked(){o.motion.transitions.push(ClipTransition{from:o.clip_name.clone(),to:names.first().cloned().unwrap_or_default(),..Default::default()});}
  let mut remove=None;let mut swap=None;
  for (i,r) in o.motion.transitions.iter_mut().enumerate(){ui.push_id(i,|ui|{ui.horizontal_wrapped(|ui|{
   egui::ComboBox::from_id_salt("from").selected_text(&r.from).show_ui(ui,|ui|{ui.selectable_value(&mut r.from,"*".into(),"Any");for n in &names{ui.selectable_value(&mut r.from,n.clone(),n);}});ui.label("→");
   egui::ComboBox::from_id_salt("to").selected_text(&r.to).show_ui(ui,|ui|{for n in &names{ui.selectable_value(&mut r.to,n.clone(),n);}});
   egui::ComboBox::from_id_salt("condition").selected_text(format!("{:?}",r.condition)).show_ui(ui,|ui|{for c in [ClipCondition::Idle,ClipCondition::Moving,ClipCondition::JumpInput,ClipCondition::Finished,ClipCondition::VariablePositive]{ui.selectable_value(&mut r.condition,c,format!("{c:?}"));}});
   if ui.small_button("↑").clicked()&&i>0{swap=Some(i);}if ui.small_button("×").clicked(){remove=Some(i);}
  });if r.condition==ClipCondition::VariablePositive{ui.text_edit_singleline(&mut r.variable);}if !names.contains(&r.to){ui.colored_label(egui::Color32::LIGHT_RED,"Missing target clip");}});}
  if let Some(i)=swap{o.motion.transitions.swap(i,i-1);}if let Some(i)=remove{o.motion.transitions.remove(i);}
 });
 ui.collapsing("Frame timing / Тривалість кадрів",|ui|{let n=clip_frame_count(&o.animation);let start=o.animation.start.min(o.animation.columns.max(1).saturating_mul(o.animation.rows.max(1))-1);let i=o.animation.frame(*time).saturating_sub(start);ui.label(format!("Frame {} / {} · {:.3} s total",i+1,n,frame_start(&o.animation,n)));let mut ms=((frame_start(&o.animation,i+1)-frame_start(&o.animation,i))*1000.).round().max(1.) as u32;if ui.add(egui::DragValue::new(&mut ms).range(1..=60000).suffix(" ms")).changed(){set_frame_duration(&mut o.animation,i,ms);*time=frame_start(&o.animation,i);}if ui.button("Reset to uniform FPS").clicked(){o.animation.durations.clear();}ui.label("Store clip to persist timing in the named clip.");});
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
