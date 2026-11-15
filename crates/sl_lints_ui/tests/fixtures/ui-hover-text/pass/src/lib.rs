use egui::Button;

pub fn ui(ui: &mut egui::Ui) {
    // a glyph button with a tooltip
    let _ = ui.button("\u{1F5D1}").on_hover_text("Delete");
    // a real word label: never an icon
    let _ = ui.button("Delete");
    // the glyph is wrapped in `add`, the tooltip still covers it
    let _ = ui.add(Button::new("\u{2705}")).on_hover_text("Done");
}
