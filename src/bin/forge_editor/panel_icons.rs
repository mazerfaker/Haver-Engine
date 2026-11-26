use super::*;
/// Resolution-independent line icons; header also serves as docking drag source.
pub(super) fn header(ui:&mut egui::Ui,index:usize,title:&str){
 ui.horizontal(|ui|{
  let (r,_)=ui.allocate_exact_size(egui::vec2(18.,18.),egui::Sense::hover());
  let p=ui.painter();let stroke=egui::Stroke::new(1.5,ui.visuals().text_color());let pt=|x:f32,y:f32|r.min+egui::vec2(x,y);
  let lines: &[[f32;4]]=match index{0=>&[[4.,3.,4.,15.],[4.,6.,13.,6.],[4.,12.,13.,12.],[13.,4.,13.,8.],[13.,10.,13.,14.]],1=>&[[3.,4.,15.,4.],[3.,9.,15.,9.],[3.,14.,15.,14.],[6.,2.,6.,6.],[11.,7.,11.,11.],[8.,12.,8.,16.]],_=>&[[2.,5.,16.,5.],[2.,5.,2.,15.],[16.,5.,16.,15.],[2.,15.,16.,15.],[2.,5.,6.,2.],[6.,2.,10.,2.],[10.,2.,12.,5.]]};
  for &[a,b,c,d] in lines{p.line_segment([pt(a,b),pt(c,d)],stroke);}
  let response=ui.add(egui::Label::new(egui::RichText::new(title).strong()).sense(egui::Sense::click_and_drag()));response.dnd_set_drag_payload(index);
 });
}
