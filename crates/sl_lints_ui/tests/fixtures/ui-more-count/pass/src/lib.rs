/// Shows the first rows; 3 more go under the fold
pub fn ui(ui: &mut egui::Ui, rest: usize) {
    ui.collapsing(format!("{rest} devices"), |ui| {
        ui.label("Show more");
    });
    ui.label("more options");
}

pub fn rest(ui: &mut egui::Ui, n: usize) {
    ve_basics::fold_rest(ui, "rest", egui::RichText::new(format!("{n} more subagents, not busy")), |_| {});
}
