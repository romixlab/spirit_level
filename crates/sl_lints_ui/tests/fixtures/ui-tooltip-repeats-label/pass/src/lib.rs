use egui::Button;

pub fn ui(ui: &mut egui::Ui) {
    // the tooltip adds the shortcut: no repeat
    let _ = ui.button("Save").on_hover_text("Ctrl+S");
    // no tooltip at all
    let _ = ui.button("Delete");
    // differs only in case: not an exact match
    let _ = ui.add(Button::new("Save")).on_hover_text("save");
    // the tooltip's text matches an unrelated control's label from an earlier statement, not this one's
    let _ = ui.button("Save");
    let _ = ui.label("Delete").on_hover_text("Save");
}
