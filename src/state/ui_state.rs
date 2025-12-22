use super::AppMode;
use egui;

pub struct UiState {
    pub cursor_pos: Option<egui::Pos2>,
    pub show_export_dialog: bool,
    pub export_format: String,
    pub mode: AppMode,
    pub pan_offset: egui::Vec2,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            cursor_pos: None,
            show_export_dialog: false,
            export_format: "PNG".to_string(),
            mode: AppMode::Drawing,
            pan_offset: egui::Vec2::ZERO,
        }
    }
}
