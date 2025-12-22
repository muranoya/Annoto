use crate::drawing_tool::DrawingTool;
use egui;

pub struct DrawingState {
    pub zoom: f32,
    pub current_tool: DrawingTool,
    pub drag_start: Option<egui::Pos2>,
    pub stroke_width: f32,
    pub stroke_color: egui::Color32,
    pub fill_color: egui::Color32,
    pub rounding: u8,
    pub mosaic_granularity: u8,
}

impl Default for DrawingState {
    fn default() -> Self {
        Self {
            zoom: 100.0,
            current_tool: DrawingTool::StrokeRect,
            drag_start: None,
            stroke_width: 3.0,
            stroke_color: egui::Color32::RED,
            fill_color: egui::Color32::from_rgba_premultiplied(255, 0, 0, 128),
            rounding: 0,
            mosaic_granularity: 10,
        }
    }
}
