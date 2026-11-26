#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use forge_engine::{Object, Project, Simulation, Body, NodeKind, project_point, unproject_point};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;
use bevy::render::{camera::RenderTarget, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, render_asset::RenderAssetUsages};
use bevy_egui::{EguiGlobalSettings, PrimaryEguiContext};
#[path = "forge_editor/viewport.rs"]
mod viewport;
use viewport::*;
#[path = "forge_editor/tools.rs"]
mod tools;
use tools::*;
#[path = "forge_editor/pro.rs"]
mod pro;
use pro::Pro;

#[derive(Resource)]
struct Editor {
    project: Project, path: Option<PathBuf>, selected: Option<Uuid>, undo: Vec<Project>, redo: Vec<Project>,
    saved: Project, play: Option<Project>, zoom: f32, pan: egui::Vec2, snap: bool, uk: bool,
    sim: Simulation, tab: usize, gizmo: usize, preview: f32, stamps: HashMap<String, std::time::SystemTime>, status: String, textures: HashMap<String, egui::TextureHandle>, drag_start: Option<Project>, preview_play: bool, native: bool, view_size: [u32;2], tools: EditorTools, pro: Pro,
}
impl Default for Editor {
    fn default() -> Self { let p = Project::default(); Self { saved: p.clone(), project: p, path: None, selected: None, undo: vec![], redo: vec![], play: None, zoom: 1., pan: egui::Vec2::ZERO, snap: true, uk: true, sim: Simulation::default(), tab: 0, gizmo: 0, preview: 0., stamps: HashMap::new(), status: "Forge Engine 0.3.2 · Ready".into(), textures: HashMap::new(), drag_start: None, preview_play: false, native: true, view_size: [640,480], tools: EditorTools::default(), pro: Pro::default() } }
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
    fn open(&mut self) { if !self.confirm_discard(){return;} if let Some(path)=rfd::FileDialog::new().add_filter("Forge JSON", &["json"]).pick_file(){ match Project::load(&path){ Ok(p)=>{self.project=p.clone(); self.saved=p; self.path=Some(path); self.selected=None; self.pro.multi.clear(); self.pro.last_scan=None; self.undo.clear(); self.redo.clear(); self.textures.clear(); self.stamps.clear(); self.preview=0.; self.preview_play=false; self.status="Opened".into();},Err(e)=>self.status=e } } }
    fn import(&mut self){ if self.path.is_none(){self.save(false);} let Some(src)=rfd::FileDialog::new().add_filter("Image", &["png","jpg","jpeg"]).pick_file() else{return;}; self.import_path(src); }
    fn import_path(&mut self,src:PathBuf){if self.path.is_none(){self.save(false);} let Some(root)=self.path.as_ref().and_then(|p|p.parent()).map(|p|p.to_path_buf()) else{return;}; let result=(||->Result<Object,String>{ let img=image::open(&src).map_err(|e|e.to_string())?; let ext=src.extension().and_then(|x|x.to_str()).unwrap_or("png"); let rel=format!("assets/{}.{}",Uuid::new_v4(),ext); std::fs::create_dir_all(root.join("assets")).map_err(|e|e.to_string())?; std::fs::copy(&src,root.join(&rel)).map_err(|e|e.to_string())?; let mut o=Object::new(src.file_stem().and_then(|x|x.to_str()).unwrap_or("Sprite"),[0.,0.],[img.width() as f32,img.height() as f32],[255;4]); o.sprite=Some(rel); Ok(o)})(); match result {Ok(o)=>{self.selected=Some(o.id);self.project.objects.push(o);},Err(e)=>self.status=e} }
}
fn main(){ App::new().add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin{primary_window:Some(Window{title:"Forge Engine 0.3.2 — Native Editor".into(),resolution:(1360.,850.).into(),..default()}),..default()})).add_plugins(EguiPlugin::default()).init_resource::<Editor>().add_systems(Startup,setup).add_systems(EguiPrimaryContextPass,editor_ui).add_systems(PostUpdate,sync_viewport.before(bevy::transform::TransformSystem::TransformPropagate)).run(); }
fn setup(mut commands:Commands, mut settings:ResMut<EguiGlobalSettings>, mut images:ResMut<Assets<Image>>){settings.auto_create_primary_context=false; commands.spawn((Camera2d,PrimaryEguiContext,bevy::render::view::RenderLayers::layer(1))); let mut image=Image::new_fill(Extent3d{width:640,height:480,depth_or_array_layers:1},TextureDimension::D2,&[22,26,32,255],TextureFormat::Bgra8UnormSrgb,RenderAssetUsages::default()); image.texture_descriptor.usage=TextureUsages::TEXTURE_BINDING|TextureUsages::COPY_DST|TextureUsages::RENDER_ATTACHMENT; let target=images.add(image); commands.spawn((Camera2d,Camera{order:-1,target:RenderTarget::Image(target.clone().into()),clear_color:ClearColorConfig::Custom(Color::srgb_u8(22,26,32)),..default()},SceneCamera)); commands.insert_resource(ViewTarget(target));}
fn editor_ui(mut contexts:EguiContexts,mut editor:ResMut<Editor>, target:Res<ViewTarget>){
    let texture=contexts.add_image(target.0.clone());
    let Ok(ctx)=contexts.ctx_mut() else {return;};
    let e=&mut *editor; pro::tick(e,ctx); pro::workspace(ctx,e); if e.preview_play {e.preview+=ctx.input(|i|i.stable_dt);ctx.request_repaint();} let uk=e.uk; let playing=e.play.is_some(); let before=e.project.clone(); let mut reset_history=false; let mut history_action=false;
    let mut visuals=egui::Visuals::dark();visuals.panel_fill=egui::Color32::from_rgb(29,33,41);visuals.window_fill=egui::Color32::from_rgb(35,40,49);visuals.selection.bg_fill=egui::Color32::from_rgb(48,82,122);visuals.widgets.inactive.bg_fill=egui::Color32::from_rgb(41,47,57);visuals.widgets.hovered.bg_fill=egui::Color32::from_rgb(57,70,88);ctx.set_visuals(visuals);
    if !playing {for file in ctx.input(|i|i.raw.dropped_files.clone()){if let Some(path)=file.path{e.import_path(path);}}}
    if let Some(root)=e.path.as_ref().and_then(|p|p.parent()){for (rel,stamp) in &mut e.stamps{if let Ok(t)=std::fs::metadata(root.join(rel)).and_then(|m|m.modified()){if t!=*stamp{*stamp=t;e.textures.remove(rel);e.textures.remove(&format!("thumb:{rel}"));}}}}
    ctx.request_repaint_after(std::time::Duration::from_millis(250));
    egui::TopBottomPanel::top("toolbar").show(ctx,|ui|{ui.horizontal(|ui|{
        ui.heading("FORGE / 0.3.2"); if ui.button("Preferences").clicked(){e.pro.settings=true;} ui.separator();
        ui.menu_button(tr(uk,"Вигляд","View"),|ui|{ui.checkbox(&mut e.tools.scene,"Scene");ui.checkbox(&mut e.tools.inspector,"Inspector");ui.checkbox(&mut e.tools.bottom,"Workspace");ui.checkbox(&mut e.tools.grid,"Grid");ui.checkbox(&mut e.tools.colliders,"Colliders");if ui.button("Reset panel visibility").clicked(){e.tools=EditorTools::default();}});
        ui.add_enabled_ui(!playing,|ui|{
            if ui.button(tr(uk,"Новий","New")).clicked() && e.confirm_discard(){e.project=Project::default();e.saved=e.project.clone();e.path=None;e.selected=None;e.pro.multi.clear();e.textures.clear();e.undo.clear();e.redo.clear();reset_history=true;}
            if ui.button(tr(uk,"Відкрити","Open")).clicked(){e.open();reset_history=true;}
            if ui.button(tr(uk,"Зберегти","Save")).clicked(){e.save(false);}
            if ui.button(tr(uk,"Зберегти як","Save as")).clicked(){e.save(true);}
            if ui.button("↶").clicked(){e.undo();history_action=true;} if ui.button("↷").clicked(){e.redo();history_action=true;}
        }); ui.separator();
        if ui.button(if playing {"■ Stop"}else{"▶ Play"}).clicked(){if playing{e.play=None;}else{e.sim=Simulation::default();e.play=Some(e.project.clone());}}
        if ui.button(if uk{"EN"}else{"UA"}).clicked(){e.uk=!uk;}
        ui.label(format!("{}{}",e.project.name,if e.saved!=e.project{" *"}else{""}));
    });});
    egui::TopBottomPanel::bottom("status").show(ctx,|ui|{ui.horizontal(|ui|{ui.label(&e.status);ui.separator();ui.label(format!("Rust / Bevy · Native GPU | {:.0} FPS | {} objects",e.pro.fps,e.project.objects.len()));});});
    if e.tools.scene { egui::SidePanel::left("scene").resizable(true).default_width(220.).show(ctx,|ui|{
        ui.heading(tr(uk,"Сцена","Scene")); ui.add(egui::TextEdit::singleline(&mut e.tools.search).hint_text(tr(uk,"Пошук об’єктів","Search objects"))); ui.add_enabled(!playing,egui::Checkbox::new(&mut e.project.isometric,"Isometric 2.5D")); ui.separator();
        ui.add_enabled_ui(!playing,|ui|{
            ui.horizontal(|ui|{if ui.button("+ Object").clicked(){let o=Object::new("Object",[0.,0.],[64.,64.],[150,170,190,255]);e.selected=Some(o.id);e.project.objects.push(o);} if ui.button("−").clicked(){delete_selected(e);}});
            egui::ScrollArea::vertical().show(ui,|ui|{if !e.tools.search.is_empty(){for o in &e.project.objects {if o.name.to_lowercase().contains(&e.tools.search.to_lowercase()) && ui.selectable_label(e.selected==Some(o.id),&o.name).clicked(){e.selected=Some(o.id);}} return;} let mut visited=std::collections::HashSet::new(); scene_tree(ui,&e.project.objects,None,&mut e.selected,&mut visited); for o in &e.project.objects {if !visited.contains(&o.id) && ui.selectable_label(e.selected==Some(o.id),format!("◇ {} (orphan)",o.name)).clicked(){e.selected=Some(o.id);}}});
        });
    });
    }
    if e.tools.inspector { egui::SidePanel::right("inspector").resizable(true).default_width(265.).show(ctx,|ui|{
        ui.heading(tr(uk,"Інспектор","Inspector")); ui.separator(); if !playing {ui.horizontal(|ui|{if ui.button("Duplicate").clicked(){duplicate(e);}if ui.button("Focus").clicked(){focus(e);}});} ui.add_enabled_ui(!playing,|ui|{
            ui.label(tr(uk,"Назва проєкту","Project name"));ui.text_edit_singleline(&mut e.project.name);ui.separator();
            let parents=e.project.objects.iter().filter(|o|valid_parent(&e.project,e.selected,o.id)).map(|o|(o.id,o.name.clone())).collect::<Vec<_>>();
            if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){
                egui::ComboBox::from_id_salt("parent").selected_text(o.parent.and_then(|id|parents.iter().find(|p|p.0==id).map(|p|p.1.clone())).unwrap_or("Root".into())).show_ui(ui,|ui|{ui.selectable_value(&mut o.parent,None,"Root");for (id,name) in &parents{if *id!=o.id{ui.selectable_value(&mut o.parent,Some(*id),name);}}});
                ui.checkbox(&mut o.camera,"Camera2D");egui::ComboBox::from_id_salt("body").selected_text(format!("{:?}",o.body)).show_ui(ui,|ui|{for b in [Body::None,Body::Static,Body::Dynamic,Body::Trigger]{ui.selectable_value(&mut o.body,b,format!("{:?}",b));}});
                ui.text_edit_singleline(&mut o.name); ui.collapsing("Transform",|ui|{for i in 0..2 {ui.horizontal(|ui|{ui.label(["X","Y"][i]);ui.add(egui::DragValue::new(&mut o.position[i]));});}for i in 0..2{ui.horizontal(|ui|{ui.label(["W","H"][i]);ui.add(egui::DragValue::new(&mut o.size[i]).range(1. ..=8192.));});}ui.add(egui::DragValue::new(&mut o.rotation).prefix("Angle ° "));ui.add(egui::DragValue::new(&mut o.layer).prefix("Layer "));});
                ui.collapsing("Sprite",|ui|{ui.color_edit_button_srgba_unmultiplied(&mut o.color);ui.label(o.sprite.as_deref().unwrap_or("Solid color"));});ui.checkbox(&mut o.player,tr(uk,"Керування WASD","WASD controller"));
            } else {ui.label(tr(uk,"Вибери об’єкт","Select an object"));}
        });
    });
    }
    if e.tools.bottom { egui::TopBottomPanel::bottom("assets").resizable(true).default_height(200.).show(ctx,|ui|{
        ui.horizontal(|ui|{for (i,t) in ["Assets","Animation","Lua","Visual Script","Console"].iter().enumerate(){ui.selectable_value(&mut e.tab,i,*t);}});ui.separator();
        ui.add_enabled_ui(!playing,|ui|{match e.tab{
        0=>{pro::assets(ui,e,ctx);},
        1=>{if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){ui.horizontal(|ui|{if ui.button(if e.preview_play{"Pause"}else{"Preview ▶"}).clicked(){e.preview_play=!e.preview_play;}if ui.button("Rewind").clicked(){e.preview=0.;e.preview_play=false;}if ui.button("Store clip").clicked(){o.clips.insert(o.clip_name.clone(),o.animation.clone());} egui::ComboBox::from_id_salt("clips").selected_text(&o.clip_name).show_ui(ui,|ui|{for name in o.clips.keys(){ui.selectable_value(&mut o.clip_name,name.clone(),name);}});if ui.button("Load clip").clicked(){if let Some(a)=o.clips.get(&o.clip_name){o.animation=a.clone();e.preview=0.;}}ui.text_edit_singleline(&mut o.clip_name);});let a=&mut o.animation;pro::timeline(ui,a,&mut e.preview);ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut a.columns).range(1..=128).prefix("Columns "));ui.add(egui::DragValue::new(&mut a.rows).range(1..=128).prefix("Rows "));if ui.button("Slice all").clicked(){a.start=0;a.frames=a.columns*a.rows;}ui.add(egui::DragValue::new(&mut a.start).range(0..=a.columns.saturating_mul(a.rows).saturating_sub(1)).prefix("Start "));ui.add(egui::DragValue::new(&mut a.frames).range(1..=a.columns.saturating_mul(a.rows).max(1)).prefix("Frames "));ui.add(egui::DragValue::new(&mut a.fps).range(1. ..=120.).prefix("FPS "));ui.checkbox(&mut a.looping,"Loop");});ui.add(egui::Slider::new(&mut e.preview,0. ..=a.frames as f32/a.fps.max(1.)).text("Timeline (s)"));ui.horizontal(|ui|{if ui.button("Previous frame").clicked(){e.preview_play=false;e.preview=(e.preview-1./a.fps.max(1.)).max(0.);}if ui.button("Next frame").clicked(){e.preview_play=false;e.preview=(e.preview+1./a.fps.max(1.)).min((a.frames.saturating_sub(1)) as f32/a.fps.max(1.));}ui.label(format!("Frame {}",a.frame(e.preview)+1));}); if let Some(tex)=o.sprite.as_ref().and_then(|rel|e.textures.get(rel)){let uv=a.uv(e.preview);ui.add(egui::Image::new((tex.id(),egui::vec2(72.,72.))).uv(egui::Rect::from_min_max(egui::pos2(uv[0][0],uv[0][1]),egui::pos2(uv[2][0],uv[2][1]))));}}},
        2=>{if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){if ui.button("Insert template").clicked(){o.script="function update(dt)\n  x = x + input_x * 100 * dt\nend\nfunction on_trigger(name)\nend".into();}ui.add(egui::TextEdit::multiline(&mut o.script).code_editor().desired_rows(6).desired_width(f32::INFINITY));}},
        3=>{if let Some(o)=e.project.objects.iter_mut().find(|o|Some(o.id)==e.selected){ui.horizontal(|ui|{for n in [NodeKind::Update,NodeKind::Trigger,NodeKind::MoveRight,NodeKind::MoveUp,NodeKind::Jump]{if ui.button(format!("+ {:?}",n)).clicked(){o.graph.nodes.push(n);}}if ui.button("Clear").clicked(){o.graph=Default::default();}});egui::ScrollArea::both().show(ui,|ui|{let n=o.graph.nodes.len();for i in 0..n{ui.horizontal(|ui|{ui.label(format!("{}: {:?}",i,o.graph.nodes[i]));for j in 0..n{if i!=j{let edge=[i,j];let mut on=o.graph.edges.contains(&edge);if ui.checkbox(&mut on,format!("→{j}")).changed(){if on{o.graph.edges.push(edge);}else{o.graph.edges.retain(|x|*x!=edge);}}}}});}});}},
        _=>{for err in e.sim.errors.iter().rev().take(5){ui.label(err);}}
        }});if playing && e.tab==4{for err in e.sim.errors.iter().rev().take(5){ui.label(err);}}
    });
    }
    egui::CentralPanel::default().show(ctx,|ui|{
        ui.horizontal(|ui|{ui.strong(if playing{"GAME / WASD"}else{"2D SCENE"});ui.checkbox(&mut e.snap,"Snap");ui.add(egui::DragValue::new(&mut e.tools.snap).range(1. ..=256.).suffix(" px"));ui.checkbox(&mut e.native,"GPU"); egui::ComboBox::from_id_salt("axis").selected_text(["XY","X","Y"][e.tools.axis]).show_ui(ui,|ui|{for (i,t) in ["XY","X","Y"].iter().enumerate(){ui.selectable_value(&mut e.tools.axis,i,*t);}});for (i,t) in ["Move","Rotate","Scale"].iter().enumerate(){ui.selectable_value(&mut e.gizmo,i,*t);}ui.add(egui::Slider::new(&mut e.zoom,0.2..=4.).text("Zoom"));if ui.button("Reset view").clicked(){e.pan=egui::Vec2::ZERO;e.zoom=1.;}});
        let (rect,response)=ui.allocate_exact_size(ui.available_size(),egui::Sense::click_and_drag()); let painter=ui.painter_at(rect);painter.rect_filled(rect,0.,egui::Color32::from_rgb(22,26,32)); e.view_size=[rect.width().round().clamp(1.,4096.) as u32,rect.height().round().clamp(1.,4096.) as u32]; if e.native {painter.image(texture,rect,egui::Rect::from_min_max(egui::pos2(0.,0.),egui::pos2(1.,1.)),egui::Color32::WHITE);}
        if response.hovered(){let scroll=ctx.input(|i|i.smooth_scroll_delta.y);let old=e.zoom;e.zoom=(e.zoom*(1.+scroll*0.001)).clamp(0.2,4.);if let Some(p)=response.hover_pos(){let offset=p-rect.center();e.pan=offset-(offset-e.pan)*(e.zoom/old);}}
        if response.dragged_by(egui::PointerButton::Middle){e.pan+=ctx.input(|i|i.pointer.delta());}
        let origin=rect.center()+e.pan;let z=e.zoom;let step=(e.tools.snap*z).max(8.);
        if e.tools.grid && !e.project.isometric {
        let mut x=rect.left()+(origin.x-rect.left()).rem_euclid(step);while x<rect.right(){painter.line_segment([egui::pos2(x,rect.top()),egui::pos2(x,rect.bottom())],egui::Stroke::new(1.,egui::Color32::from_gray(37)));x+=step;}
        let mut y=rect.top()+(origin.y-rect.top()).rem_euclid(step);while y<rect.bottom(){painter.line_segment([egui::pos2(rect.left(),y),egui::pos2(rect.right(),y)],egui::Stroke::new(1.,egui::Color32::from_gray(37)));y+=step;}
        }
        if e.tools.grid && e.project.isometric {iso_grid(&painter,rect,origin,z);}
        if let Some(p)=e.play.as_mut(){let dt=ctx.input(|i|i.stable_dt);let d=ctx.input(|i|[(i.key_down(egui::Key::D) as i32-i.key_down(egui::Key::A) as i32) as f32,(i.key_down(egui::Key::W) as i32-i.key_down(egui::Key::S) as i32) as f32]);e.sim.step(p,dt,d);ctx.request_repaint();}
        let mut objects=e.play.as_ref().unwrap_or(&e.project).objects.clone();let iso=e.project.isometric;objects.sort_by(|a,b|a.layer.cmp(&b.layer).then_with(||if iso{project_point(b.position,true)[1].total_cmp(&project_point(a.position,true)[1])}else{std::cmp::Ordering::Equal}));
        let center=|o:&Object|{let p=project_point(o.position,iso);origin+egui::vec2(p[0]*z,-p[1]*z)};
        let shape=|o:&Object|{let c=center(o);let a=-o.rotation.to_radians();let (s,co)=a.sin_cos();let w=o.size[0]*z/2.;let h=o.size[1]*z/2.;[[-w,-h],[w,-h],[w,h],[-w,h]].map(|p|c+egui::vec2(p[0]*co-p[1]*s,p[0]*s+p[1]*co))};
        let hit=|o:&Object,p:egui::Pos2|{let d=p-center(o);let (s,c)=o.rotation.to_radians().sin_cos();(d.x*c-d.y*s).abs()<=o.size[0]*z/2. && (d.x*s+d.y*c).abs()<=o.size[1]*z/2.};
        if !playing && (response.clicked()||response.drag_started_by(egui::PointerButton::Primary)){if let Some(p)=response.interact_pointer_pos(){let found=objects.iter().rev().find(|o|hit(o,p)).map(|o|o.id);if ctx.input(|i|i.modifiers.shift){if let Some(id)=found{if !e.pro.multi.insert(id){e.pro.multi.remove(&id);}e.selected=Some(id);}}else{e.pro.multi.clear();e.selected=found;}}if response.drag_started(){e.drag_start=Some(e.project.clone());}}
        if !playing && response.dragged_by(egui::PointerButton::Primary){let mut d=ctx.input(|i|i.pointer.delta()); if e.tools.axis==1{d.y=0.;}else if e.tools.axis==2{d.x=0.;} for o in e.project.objects.iter_mut().filter(|o|Some(o.id)==e.selected||e.pro.multi.contains(&o.id)){match e.gizmo{1=>o.rotation+=d.x,2=>{o.size[0]=(o.size[0]+d.x/z).max(1.);o.size[1]=(o.size[1]-d.y/z).max(1.);},_=>{let p=unproject_point([d.x/z,-d.y/z],iso);o.position[0]+=p[0];o.position[1]+=p[1];}}}}
        if response.drag_stopped(){if e.snap && e.gizmo==0{for o in e.project.objects.iter_mut().filter(|o|Some(o.id)==e.selected||e.pro.multi.contains(&o.id)){if e.drag_start.is_some(){for v in &mut o.position{*v=(*v/e.tools.snap).round()*e.tools.snap;}}}}}
        for o in &objects{let pts=shape(o);let color=egui::Color32::from_rgba_unmultiplied(o.color[0],o.color[1],o.color[2],o.color[3]);let mut drawn=false;
            if let Some(rel)=&o.sprite{if !e.textures.contains_key(rel){if let Some(root)=e.path.as_ref().and_then(|p|p.parent()){if let Ok(img)=image::open(root.join(rel)){let rgba=img.to_rgba8();let tex=ctx.load_texture(rel,egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize,rgba.height() as usize],rgba.as_raw()),egui::TextureOptions::NEAREST);e.textures.insert(rel.clone(),tex);if let Ok(t)=std::fs::metadata(root.join(rel)).and_then(|m|m.modified()){e.stamps.insert(rel.clone(),t);}}}}if let Some(tex)=e.textures.get(rel){let mut mesh=egui::Mesh::with_texture(tex.id());for (p,uv) in pts.iter().zip(o.animation.uv(if playing{e.sim.elapsed}else{e.preview})){mesh.vertices.push(egui::epaint::Vertex{pos:*p,uv:egui::pos2(uv[0],uv[1]),color});}mesh.indices.extend([0,1,2,0,2,3]);if !e.native {painter.add(egui::Shape::mesh(mesh));}drawn=true;}}
            if o.camera && !playing {painter.add(egui::Shape::closed_line(pts.to_vec(),egui::Stroke::new(1.,egui::Color32::LIGHT_GREEN)));painter.text(center(o),egui::Align2::CENTER_CENTER,"Camera",egui::FontId::monospace(12.),egui::Color32::LIGHT_GREEN);} if e.tools.colliders && o.body!=Body::None {painter.add(egui::Shape::closed_line(pts.to_vec(),egui::Stroke::new(1.,egui::Color32::YELLOW)));} if !drawn && !e.native{painter.add(egui::Shape::convex_polygon(pts.to_vec(),color,egui::Stroke::NONE));}if !playing && (e.selected==Some(o.id)||e.pro.multi.contains(&o.id)){painter.add(egui::Shape::closed_line(pts.to_vec(),egui::Stroke::new(2.,egui::Color32::LIGHT_BLUE)));let c=center(o);painter.line_segment([c,c+egui::vec2(45.,0.)],egui::Stroke::new(2.,egui::Color32::RED));painter.line_segment([c,c+egui::vec2(0.,-45.)],egui::Stroke::new(2.,egui::Color32::GREEN));}
        }
    });
    if !playing && !ctx.wants_keyboard_input(){if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::C)){pro::copy(e);}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::V)){pro::paste(e);}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::G)){pro::group(e);}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::S)){e.save(false);}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::Z)){e.undo();history_action=true;}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::Y)){e.redo();history_action=true;}if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::D)){duplicate(e);}if pro::key(ctx,&e.pro.keys.focus){focus(e);}if pro::key(ctx,&e.pro.keys.move_key){e.gizmo=0;}if pro::key(ctx,&e.pro.keys.rotate){e.gizmo=1;}if pro::key(ctx,&e.pro.keys.scale){e.gizmo=2;}if ctx.input(|i|i.key_pressed(egui::Key::Delete)){delete_selected(e);}}
    if e.drag_start.is_some() && !ctx.input(|i|i.pointer.primary_down()){if let Some(old)=e.drag_start.take(){e.record(old);}}else if e.drag_start.is_none()&&!reset_history&&!history_action{e.record(before);}
}

fn scene_tree(ui:&mut egui::Ui,objects:&[Object],parent:Option<Uuid>,selected:&mut Option<Uuid>,visited:&mut std::collections::HashSet<Uuid>){for o in objects.iter().filter(|o|o.parent==parent){if !visited.insert(o.id){continue;}let has=objects.iter().any(|c|c.parent==Some(o.id));if has{egui::CollapsingHeader::new(format!("◇ {}",o.name)).id_salt(o.id).default_open(true).show(ui,|ui|{if ui.selectable_label(*selected==Some(o.id),"Select parent").clicked(){*selected=Some(o.id);}scene_tree(ui,objects,Some(o.id),selected,visited);});}else if ui.selectable_label(*selected==Some(o.id),format!("◇ {}",o.name)).clicked(){*selected=Some(o.id);}}}
