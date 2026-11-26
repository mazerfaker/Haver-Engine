#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use forge_engine::{Object, Project};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

#[derive(Resource)]
struct Editor {
    project: Project, path: Option<PathBuf>, selected: Option<Uuid>, undo: Vec<Project>, redo: Vec<Project>,
    saved: Project, play: Option<Project>, zoom: f32, pan: egui::Vec2, snap: bool, uk: bool,
    status: String, textures: HashMap<String, egui::TextureHandle>, drag_start: Option<Project>,
}
impl Default for Editor {
    fn default() -> Self { let p = Project::default(); Self { saved: p.clone(), project: p, path: None, selected: None, undo: vec![], redo: vec![], play: None, zoom: 1., pan: egui::Vec2::ZERO, snap: true, uk: true, status: "Forge Engine 0.1 · Ready".into(), textures: HashMap::new(), drag_start: None } }
}
fn tr(uk: bool, a: &'static str, b: &'static str) -> &'static str { if uk { a } else { b } }
impl Editor {
    fn record(&mut self, old: Project) { if old != self.project { self.undo.push(old); if self.undo.len()>100 { self.undo.remove(0); } self.redo.clear(); } }
    fn undo(&mut self) { if let Some(p)=self.undo.pop(){self.redo.push(std::mem::replace(&mut self.project,p));} }
    fn redo(&mut self) { if let Some(p)=self.redo.pop(){self.undo.push(std::mem::replace(&mut self.project,p));} }
    fn save(&mut self, as_new: bool) {
        let path = if as_new || self.path.is_none() { rfd::FileDialog::new().set_file_name("project.forge.json").save_file() } else { self.path.clone() };
        if let Some(path)=path {
            // Save As copies referenced assets to preserve relative paths.
            let result = (|| -> Result<(),String> {
                if let Some(old)=&self.path { for o in &self.project.objects { if let Some(s)=&o.sprite { let from=old.parent().unwrap().join(s); let to=path.parent().unwrap().join(s); if from!=to { std::fs::create_dir_all(to.parent().unwrap()).map_err(|e|e.to_string())?; std::fs::copy(from,to).map_err(|e|e.to_string())?; } } } }
                self.project.save(&path)
            })();
            match result { Ok(())=>{ self.path=Some(path); self.saved=self.project.clone(); self.status="Saved".into(); }, Err(e)=>self.status=e }
        }
    }
    fn confirm_discard(&self) -> bool { self.saved==self.project || matches!(rfd::MessageDialog::new().set_title("Forge Engine").set_description("Discard unsaved changes? / Відкинути незбережені зміни?").set_buttons(rfd::MessageButtons::YesNo).show(),rfd::MessageDialogResult::Yes) }
    fn open(&mut self) { if !self.confirm_discard(){return;} if let Some(path)=rfd::FileDialog::new().add_filter("Forge JSON", &["json"]).pick_file(){ match Project::load(&path){ Ok(p)=>{self.project=p.clone(); self.saved=p; self.path=Some(path); self.selected=None; self.undo.clear(); self.redo.clear(); self.textures.clear(); self.status="Opened".into();},Err(e)=>self.status=e } } }
    fn import(&mut self){ if self.path.is_none(){self.save(false);} let Some(root)=self.path.as_ref().and_then(|p|p.parent()).map(|p|p.to_path_buf()) else{return;}; let Some(src)=rfd::FileDialog::new().add_filter("Image", &["png","jpg","jpeg"]).pick_file() else{return;}; let result=(||->Result<Object,String>{ let img=image::open(&src).map_err(|e|e.to_string())?; let ext=src.extension().and_then(|x|x.to_str()).unwrap_or("png"); let rel=format!("assets/{}.{}",Uuid::new_v4(),ext); std::fs::create_dir_all(root.join("assets")).map_err(|e|e.to_string())?; std::fs::copy(&src,root.join(&rel)).map_err(|e|e.to_string())?; let mut o=Object::new(src.file_stem().and_then(|x|x.to_str()).unwrap_or("Sprite"),[0.,0.],[img.width() as f32,img.height() as f32],[255;4]); o.sprite=Some(rel); Ok(o)})(); match result {Ok(o)=>{self.selected=Some(o.id);self.project.objects.push(o);},Err(e)=>self.status=e} }
}
fn main(){ App::new().add_plugins(DefaultPlugins.set(WindowPlugin{primary_window:Some(Window{title:"Forge Engine 0.1 — Native Editor".into(),resolution:(1360.,850.).into(),..default()}),..default()})).add_plugins(EguiPlugin::default()).init_resource::<Editor>().add_systems(Startup,setup).add_systems(EguiPrimaryContextPass,editor_ui).run(); }
fn setup(mut commands:Commands){commands.spawn(Camera2d);}
fn editor_ui(mut contexts:EguiContexts,mut editor:ResMut<Editor>){
    let Ok(ctx)=contexts.ctx_mut() else {return;};
    let e=&mut *editor; let uk=e.uk; let playing=e.play.is_some(); let before=e.project.clone(); let mut reset_history=false; let mut history_action=false;
    ctx.set_visuals(egui::Visuals::dark());
    egui::TopBottomPanel::top("toolbar").show(ctx,|ui|{ui.horizontal(|ui|{
        ui.heading("FORGE / 0.1"); ui.separator();
        ui.add_enabled_ui(!playing,|ui|{
            if ui.button(tr(uk,"Новий","New")).clicked() && e.confirm_discard(){e.project=Project::default();e.saved=e.project.clone();e.path=None;e.selected=None;e.textures.clear();e.undo.clear();e.redo.clear();reset_history=true;}
            if ui.button(tr(uk,"Відкрити","Open")).clicked(){e.open();reset_history=true;}
            if ui.button(tr(uk,"Зберегти","Save")).clicked(){e.save(false);}
            if ui.button(tr(uk,"Зберегти як","Save as")).clicked(){e.save(true);}
            if ui.button("↶").clicked(){e.undo();history_action=true;} if ui.button("↷").clicked(){e.redo();history_action=true;}
        }); ui.separator();
        if ui.button(if playing {"■ Stop"}else{"▶ Play"}).clicked(){if playing{e.play=None;}else{e.play=Some(e.project.clone());}}
        if ui.button(if uk{"EN"}else{"UA"}).clicked(){e.uk=!uk;}
        ui.label(format!("{}{}",e.project.name,if e.saved!=e.project{" *"}else{""}));
    });});
    egui::TopBottomPanel::bottom("status").show(ctx,|ui|{ui.horizontal(|ui|{ui.label(&e.status);ui.separator();ui.label("Rust / Bevy · Native GPU");});});
    egui::SidePanel::left("scene").resizable(true).default_width(220.).show(ctx,|ui|{
        ui.heading(tr(uk,"Сцена","Scene")); ui.separator();
        ui.add_enabled_ui(!playing,|ui|{
            ui.horizontal(|ui|{if ui.button("+ Object").clicked(){let o=Object::new("Object",[0.,0.],[64.,64.],[150,170,190,255]);e.selected=Some(o.id);e.project.objects.push(o);} if ui.button("−").clicked(){e.project.objects.retain(|o|Some(o.id)!=e.selected);e.selected=None;}});
            egui::ScrollArea::vertical().show(ui,|ui|{for o in &e.project.objects{if ui.selectable_label(e.selected==Some(o.id),format!("◇ {}",o.name)).clicked(){e.selected=Some(o.id);}}});
        });
    });
    egui::SidePanel::right("inspector").resizable(true).default_width(265.).show(ctx,|ui|{
        ui.heading(tr(uk,"Інспектор","Inspector")); ui.separator(); ui.add_enabled_ui(!playing,|ui|{
            ui.label(tr(uk,"Назва проєкту","Project name"));ui.text_edit_singleline(&mut e.project.name);ui.separator();
            if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){
                ui.text_edit_singleline(&mut o.name); ui.collapsing("Transform",|ui|{for i in 0..2 {ui.horizontal(|ui|{ui.label(["X","Y"][i]);ui.add(egui::DragValue::new(&mut o.position[i]));});}for i in 0..2{ui.horizontal(|ui|{ui.label(["W","H"][i]);ui.add(egui::DragValue::new(&mut o.size[i]).range(1. ..=8192.));});}ui.add(egui::DragValue::new(&mut o.rotation).prefix("Angle ° "));ui.add(egui::DragValue::new(&mut o.layer).prefix("Layer "));});
                ui.collapsing("Sprite",|ui|{ui.color_edit_button_srgba_unmultiplied(&mut o.color);ui.label(o.sprite.as_deref().unwrap_or("Solid color"));});ui.checkbox(&mut o.player,tr(uk,"Керування WASD","WASD controller"));
            } else {ui.label(tr(uk,"Вибери об’єкт","Select an object"));}
        });
    });
    egui::TopBottomPanel::bottom("assets").resizable(true).default_height(115.).show(ctx,|ui|{ui.horizontal(|ui|{ui.heading(tr(uk,"Ресурси","Assets"));if ui.add_enabled(!playing,egui::Button::new("+ PNG / JPG")).clicked(){e.import();}});ui.separator();egui::ScrollArea::horizontal().show(ui,|ui|{ui.horizontal(|ui|{for o in &e.project.objects{if o.sprite.is_some() && ui.selectable_label(e.selected==Some(o.id),&o.name).clicked(){e.selected=Some(o.id);}}});});ui.label(tr(uk,"PNG/JPG імпортуються у папку проєкту assets/","PNG/JPG are copied into project assets/"));});
    egui::CentralPanel::default().show(ctx,|ui|{
        ui.horizontal(|ui|{ui.strong(if playing{"GAME / WASD"}else{"2D SCENE"});ui.checkbox(&mut e.snap,"Snap 16");ui.add(egui::Slider::new(&mut e.zoom,0.2..=4.).text("Zoom"));if ui.button("Reset view").clicked(){e.pan=egui::Vec2::ZERO;e.zoom=1.;}});
        let (rect,response)=ui.allocate_exact_size(ui.available_size(),egui::Sense::click_and_drag()); let painter=ui.painter_at(rect);painter.rect_filled(rect,0.,egui::Color32::from_rgb(22,26,32));
        if response.hovered(){let scroll=ctx.input(|i|i.smooth_scroll_delta.y);e.zoom=(e.zoom*(1.+scroll*0.001)).clamp(0.2,4.);}
        if response.dragged_by(egui::PointerButton::Middle){e.pan+=ctx.input(|i|i.pointer.delta());}
        let origin=rect.center()+e.pan;let z=e.zoom;let step=32.*z;
        let mut x=rect.left()+(origin.x-rect.left()).rem_euclid(step);while x<rect.right(){painter.line_segment([egui::pos2(x,rect.top()),egui::pos2(x,rect.bottom())],egui::Stroke::new(1.,egui::Color32::from_gray(37)));x+=step;}
        let mut y=rect.top()+(origin.y-rect.top()).rem_euclid(step);while y<rect.bottom(){painter.line_segment([egui::pos2(rect.left(),y),egui::pos2(rect.right(),y)],egui::Stroke::new(1.,egui::Color32::from_gray(37)));y+=step;}
        if let Some(p)=e.play.as_mut(){let dt=ctx.input(|i|i.stable_dt).min(0.05);let d=ctx.input(|i|[(i.key_down(egui::Key::D) as i32-i.key_down(egui::Key::A) as i32) as f32,(i.key_down(egui::Key::W) as i32-i.key_down(egui::Key::S) as i32) as f32]);let d=Vec2::from_array(d).normalize_or_zero();for o in &mut p.objects{if o.player{o.position[0]+=d.x*200.*dt;o.position[1]+=d.y*200.*dt;}}ctx.request_repaint();}
        let mut objects=e.play.as_ref().unwrap_or(&e.project).objects.clone();objects.sort_by_key(|o|o.layer);
        let center=|o:&Object|origin+egui::vec2(o.position[0]*z,-o.position[1]*z);
        let shape=|o:&Object|{let c=center(o);let a=-o.rotation.to_radians();let (s,co)=a.sin_cos();let w=o.size[0]*z/2.;let h=o.size[1]*z/2.;[[-w,-h],[w,-h],[w,h],[-w,h]].map(|p|c+egui::vec2(p[0]*co-p[1]*s,p[0]*s+p[1]*co))};
        let hit=|o:&Object,p:egui::Pos2|{let d=p-center(o);let (s,c)=o.rotation.to_radians().sin_cos();(d.x*c-d.y*s).abs()<=o.size[0]*z/2. && (d.x*s+d.y*c).abs()<=o.size[1]*z/2.};
        if !playing && (response.clicked()||response.drag_started_by(egui::PointerButton::Primary)){if let Some(p)=response.interact_pointer_pos(){e.selected=objects.iter().rev().find(|o|hit(o,p)).map(|o|o.id);}if response.drag_started(){e.drag_start=Some(e.project.clone());}}
        if !playing && response.dragged_by(egui::PointerButton::Primary){let d=ctx.input(|i|i.pointer.delta());if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){o.position[0]+=d.x/z;o.position[1]-=d.y/z;}}
        if response.drag_stopped(){if e.snap{if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){if e.drag_start.is_some(){for v in &mut o.position{*v=(*v/16.).round()*16.;}}}}}
        for o in &objects{let pts=shape(o);let color=egui::Color32::from_rgba_unmultiplied(o.color[0],o.color[1],o.color[2],o.color[3]);let mut drawn=false;
            if let Some(rel)=&o.sprite{if !e.textures.contains_key(rel){if let Some(root)=e.path.as_ref().and_then(|p|p.parent()){if let Ok(img)=image::open(root.join(rel)){let rgba=img.to_rgba8();let tex=ctx.load_texture(rel,egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize,rgba.height() as usize],rgba.as_raw()),egui::TextureOptions::NEAREST);e.textures.insert(rel.clone(),tex);}}}if let Some(tex)=e.textures.get(rel){let mut mesh=egui::Mesh::with_texture(tex.id());for (p,uv) in pts.iter().zip([[0.,0.],[1.,0.],[1.,1.],[0.,1.]]){mesh.vertices.push(egui::epaint::Vertex{pos:*p,uv:egui::pos2(uv[0],uv[1]),color});}mesh.indices.extend([0,1,2,0,2,3]);painter.add(egui::Shape::mesh(mesh));drawn=true;}}
            if !drawn{painter.add(egui::Shape::convex_polygon(pts.to_vec(),color,egui::Stroke::NONE));}if !playing && e.selected==Some(o.id){painter.add(egui::Shape::closed_line(pts.to_vec(),egui::Stroke::new(2.,egui::Color32::LIGHT_BLUE)));}
        }
    });
    if !playing && !ctx.wants_keyboard_input(){if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::S)){e.save(false);}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::Z)){e.undo();history_action=true;}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::Y)){e.redo();history_action=true;}if ctx.input(|i|i.key_pressed(egui::Key::Delete)){e.project.objects.retain(|o|Some(o.id)!=e.selected);e.selected=None;}}
    if e.drag_start.is_some() && !ctx.input(|i|i.pointer.primary_down()){if let Some(old)=e.drag_start.take(){e.record(old);}}else if e.drag_start.is_none()&&!reset_history&&!history_action{e.record(before);}
}
