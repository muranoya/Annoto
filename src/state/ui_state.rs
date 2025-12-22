use super::AppMode;
use egui;

#[derive(Clone, Copy, Debug)]
pub struct TouchPoint {
    pub pos: egui::Pos2,
    pub id: u64,
}

pub struct UiState {
    pub cursor_pos: Option<egui::Pos2>,
    pub show_export_dialog: bool,
    pub export_format: String,
    pub mode: AppMode,
    pub pan_offset: egui::Vec2,
    // タッチ状態管理
    pub touch_points: Vec<TouchPoint>,
    pub prev_touch_points: Vec<TouchPoint>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            cursor_pos: None,
            show_export_dialog: false,
            export_format: "PNG".to_string(),
            mode: AppMode::Drawing,
            pan_offset: egui::Vec2::ZERO,
            touch_points: Vec::new(),
            prev_touch_points: Vec::new(),
        }
    }
}
