use super::*;
use forge_engine::{Widget, adaptive_widget, hud_group_path, move_hud_widget};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Studio {pub open:bool, selected:Option<usize>, preview:usize, snap:bool}
#[derive(Default)]
struct Group {members:Vec<usize>, children:BTreeMap<String,Group>}
impl Group {
    fn insert(&mut self, path:&[&str], index:usize) {
        if let Some((head,tail))=path.split_first(){self.children.entry((*head).into()).or_default().insert(tail,index);}else{self.members.push(index);}
    }
    fn draw(&self,ui:&mut egui::Ui,widgets:&[Widget],selected:&mut Option<usize>){
        for &i in &self.members {if ui.selectable_label(*selected==Some(i),format!("{} · {}",widgets[i].text,widgets[i].action)).clicked(){*selected=Some(i);}}
        for (name,child) in &self.children {ui.push_id(name,|ui|{ui.collapsing(name.as_str(),|ui|child.draw(ui,widgets,selected));});}
    }
}
fn widget_preset(action:&str)->Widget{
    Widget{text:match action{"left"=>"Left","right"=>"Right","jump"=>"Jump",_=>"Label"}.into(),action:action.into(),anchor:[0.1,0.1],..Default::default()}
}
pub(super) fn window(ctx:&egui::Context,e:&mut Editor){
    if !e.hud_studio.open{return;}
    let old=e.project.clone();
    let mut open=e.hud_studio.open;
    let studio=&mut e.hud_studio;
    let s=&mut e.project.settings;
    let mut begin_drag=false;
    egui::Window::new("UI Studio / Студія UI").id(egui::Id::new("hud_studio18")).open(&mut open).default_size([840.,550.]).resizable(true).show(ctx,|ui|{
        if studio.selected.is_some_and(|i|i>=s.widgets.len()){studio.selected=None;}
        ui.horizontal_wrapped(|ui|{
            for a in ["label","left","right","jump"]{if ui.button(format!("+ {a}")).clicked(){s.widgets.push(widget_preset(a));studio.selected=Some(s.widgets.len()-1);}}
            if ui.button("+ Inventory slots").clicked(){let start=s.widgets.len();for i in 0..6{s.widgets.push(Widget{text:format!("Slot {}",i+1),action:"label".into(),anchor:[0.05+(i%3) as f32*0.22,0.15+(i/3) as f32*0.20],size:[96.,38.],group:"HUD/Inventory".into()});}studio.selected=Some(start);}
            ui.checkbox(&mut studio.snap,"Snap 5%");
        });
        ui.checkbox(&mut s.hud_canvas_groups,"Canvas placement for grouped HUD (Player)");
        if !s.hud_canvas_groups{ui.label("Legacy mode: grouped elements are drawn in floating group windows in Player. Enable canvas placement for matching anchors.");}
        let design=s.production.resolution;
        let choices=[("Project",design),("16:9",[1280,720]),("4:3",[1024,768]),("Portrait",[720,1280])];
        studio.preview=studio.preview.min(choices.len()-1);
        ui.horizontal(|ui|{for (i,(name,_)) in choices.iter().enumerate(){ui.selectable_value(&mut studio.preview,i,*name);}});
        let resolution=choices[studio.preview].1.map(|v|v.max(1));
        ui.separator();
        ui.columns(2,|cols|{
            let mut root=Group::default();for (i,w) in s.widgets.iter().enumerate(){let path=hud_group_path(&w.group);let parts=path.split('/').filter(|p|!p.is_empty()).collect::<Vec<_>>();root.insert(&parts,i);}
            cols[0].strong("Hierarchy");
            egui::ScrollArea::vertical().id_salt("hud_tree18").max_height(320.).show(&mut cols[0],|ui|root.draw(ui,&s.widgets,&mut studio.selected));
            cols[1].strong(format!("{} × {} preview",resolution[0],resolution[1]));
            let width=cols[1].available_width().clamp(80.,640.);
            let ratio=resolution[0] as f32/resolution[1] as f32;
            let size=egui::vec2(width.min(320.*ratio), (width/ratio).min(320.));
            let (rect,_)=cols[1].allocate_exact_size(size,egui::Sense::hover());
            let painter=cols[1].painter_at(rect);
            painter.rect_filled(rect,3.,egui::Color32::from_rgb(20,25,33));
            for i in 1..10 {let t=i as f32/10.;let x=rect.left()+rect.width()*t;let y=rect.top()+rect.height()*t;let stroke=egui::Stroke::new(1_f32,egui::Color32::from_gray(45));painter.line_segment([egui::pos2(x,rect.top()),egui::pos2(x,rect.bottom())],stroke);painter.line_segment([egui::pos2(rect.left(),y),egui::pos2(rect.right(),y)],stroke);}
            let view=[rect.width(),rect.height()];
            for (i,w) in s.widgets.iter_mut().enumerate(){
                let [x,y,width,height]=adaptive_widget(w,view,design);
                let r=egui::Rect::from_min_size(rect.min+egui::vec2(x,y),egui::vec2(width,height));
                let response=cols[1].interact(r,egui::Id::new(("ui_studio_widget18",i)),egui::Sense::click_and_drag());
                if response.clicked()||response.drag_started(){studio.selected=Some(i);}
                if response.drag_started(){begin_drag=true;}
                if response.dragged(){let d=ctx.input(|v|v.pointer.delta());move_hud_widget(w,[d.x,d.y],view,design);}
                if response.drag_stopped()&&studio.snap{w.anchor=w.anchor.map(|v|(v*20.).round()/20.);move_hud_widget(w,[0.,0.],view,design);}
                let [x,y,width,height]=adaptive_widget(w,view,design);let r=egui::Rect::from_min_size(rect.min+egui::vec2(x,y),egui::vec2(width,height));
                painter.rect_filled(r,2.,if studio.selected==Some(i){egui::Color32::from_rgb(62,119,174)}else{egui::Color32::from_rgb(53,66,83)});
                painter.with_clip_rect(r.intersect(rect)).text(r.center(),egui::Align2::CENTER_CENTER,&w.text,egui::FontId::proportional(10.),egui::Color32::WHITE);
            }
        });
        ui.separator();
        if let Some(i)=studio.selected{
            ui.strong("Widget Inspector");
            let w=&mut s.widgets[i];
            ui.horizontal(|ui|{ui.label("Text");ui.text_edit_singleline(&mut w.text);});
            ui.horizontal(|ui|{ui.label("Group path");ui.text_edit_singleline(&mut w.group);if ui.button("Normalize").clicked(){w.group=hud_group_path(&w.group);}});
            egui::ComboBox::from_id_salt("hud_action18").selected_text(&w.action).show_ui(ui,|ui|{for a in ["label","left","right","jump"]{ui.selectable_value(&mut w.action,a.into(),a);}});
            ui.horizontal_wrapped(|ui|{for (j,v) in w.anchor.iter_mut().enumerate(){ui.add(egui::DragValue::new(v).range(0. ..=1.).speed(0.005).prefix(["X ","Y "][j]));}for (j,v) in w.size.iter_mut().enumerate(){ui.add(egui::DragValue::new(v).range(16. ..=800.).prefix(["W ","H "][j]));}});
            ui.horizontal_wrapped(|ui|{
                if ui.button("Duplicate").clicked(){let mut copy=s.widgets[i].clone();copy.anchor=copy.anchor.map(|v|(v+0.03).min(1.));s.widgets.push(copy);studio.selected=Some(s.widgets.len()-1);}
                if ui.button("Move up").clicked()&&i>0{s.widgets.swap(i,i-1);studio.selected=Some(i-1);}
                if ui.button("Move down").clicked()&&i+1<s.widgets.len(){s.widgets.swap(i,i+1);studio.selected=Some(i+1);}
                if ui.button("Remove").clicked(){s.widgets.remove(i);studio.selected=None;}
            });
        }else{ui.label("Select a widget in the hierarchy or preview.");}
        ui.label("Inventory slots are layout labels, not an inventory gameplay system.");
    });
    studio.open=open;
    if begin_drag&&e.drag_start.is_none(){e.drag_start=Some(old);}
}
