use crate::state::{DrawingState, UiState};
use egui;

pub fn render_top_panel(
    ctx: &egui::Context,
    drawing_state: &mut DrawingState,
    ui_state: &mut UiState,
    on_open_file: impl FnOnce(),
) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("ファイルを開く").clicked() {
                    on_open_file();
                }
                if ui.button("エクスポート").clicked() {
                    ui_state.show_export_dialog = true;
                }
            });
            ui.add_space(16.0);
            ui.label("倍率:");
            ui.add(
                egui::DragValue::new(&mut drawing_state.zoom)
                    .range(1.0..=500.0)
                    .suffix("%"),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(pos) = ui_state.cursor_pos {
                    ui.label(format!("X: {:.0}, Y: {:.0}", pos.x, pos.y));
                }
            });
        });
    });
}
