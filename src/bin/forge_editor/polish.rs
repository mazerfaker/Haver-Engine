use super::*;
pub(super) fn prune_selection(e: &mut Editor) {
    let ids: std::collections::HashSet<_> = e.project.objects.iter().map(|o| o.id).collect();
    e.pro.multi.retain(|id| ids.contains(id));
    if e.selected.is_some_and(|id| !ids.contains(&id)) { e.selected = None; }
}
pub(super) fn selection_label(e: &Editor) -> String {
    let count = e.pro.multi.len() + usize::from(e.selected.is_some_and(|id| !e.pro.multi.contains(&id)));
    format!("{}: {}", tr(e.uk,"Вибрано","Selected"),count)
}
fn fit_bounds(lo: egui::Vec2, hi: egui::Vec2, viewport: egui::Vec2) -> (f32, egui::Vec2) {
    let extent = (hi-lo).max(egui::vec2(1.,1.));
    let available = (viewport-egui::vec2(80.,80.)).max(egui::vec2(1.,1.));
    let zoom = (available.x/extent.x).min(available.y/extent.y).clamp(0.2,4.);
    let middle = (lo+hi)*0.5;
    (zoom, -middle*zoom)
}
pub(super) fn frame_scene(e: &mut Editor, viewport: egui::Vec2) {
    let mut lo = egui::vec2(f32::INFINITY,f32::INFINITY);
    let mut hi = egui::vec2(f32::NEG_INFINITY,f32::NEG_INFINITY);
    for o in &e.project.objects {
        let p = forge_engine::sprite_center(o,e.project.isometric);
        let center = egui::vec2(p[0],-p[1]);
        let (sin,cos)=o.rotation.to_radians().sin_cos();
        let half=egui::vec2(cos.abs()*o.size[0].abs()+sin.abs()*o.size[1].abs(),sin.abs()*o.size[0].abs()+cos.abs()*o.size[1].abs())*0.5;
        lo=lo.min(center-half); hi=hi.max(center+half);
    }
    if !lo.x.is_finite() || !hi.x.is_finite() {e.zoom=1.;e.pan=egui::Vec2::ZERO;return;}
    (e.zoom,e.pan)=fit_bounds(lo,hi,viewport);
}
pub(super) fn help(ui: &mut egui::Ui, uk: bool) {
    for (key, ua, en) in [("Home","Вмістити сцену","Frame scene"),("F","Фокус (типово)","Focus (default)"),("Esc","Зняти виділення","Clear selection"),("MMB","Панорамування","Pan"),("Scroll","Масштабування","Zoom"),("Ctrl+S","Зберегти","Save")] {
        ui.horizontal(|ui|{ui.monospace(key);ui.label(tr(uk,ua,en));});
    }
}
#[cfg(test)] mod tests {use super::*;
#[test] fn fits_and_centers(){let (z,p)=fit_bounds(egui::vec2(100.,100.),egui::vec2(300.,200.),egui::vec2(480.,280.));assert_eq!(z,2.);assert_eq!(p,egui::vec2(-400.,-300.));}
#[test] fn clears_missing_selection(){let mut e=Editor::default();let id=Uuid::new_v4();e.selected=Some(id);e.pro.multi.insert(id);prune_selection(&mut e);assert!(e.selected.is_none());assert!(e.pro.multi.is_empty());}
}
