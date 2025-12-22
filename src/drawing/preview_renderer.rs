use crate::canvas_items::*;
use crate::drawing_tool::DrawingTool;
use crate::state::DrawingState;
use egui;

pub struct PreviewRenderer;

impl PreviewRenderer {
    /// ドラッグ中のプレビューを描画
    pub fn render_drag_preview(
        ui: &mut egui::Ui,
        drawing_state: &DrawingState,
        image_rect: egui::Rect,
        scale: f32,
    ) {
        let Some(start_world) = drawing_state.drag_start else {
            return;
        };
        let Some(end_world) = ui.input(|i| i.pointer.hover_pos()) else {
            return;
        };
        if !image_rect.contains(end_world) {
            return;
        }

        match drawing_state.current_tool {
            DrawingTool::StrokeRect => {
                let min_world = egui::pos2(
                    start_world.x.min(end_world.x),
                    start_world.y.min(end_world.y),
                );
                let max_world = egui::pos2(
                    start_world.x.max(end_world.x),
                    start_world.y.max(end_world.y),
                );
                let offset_min = (min_world - image_rect.min) / scale;
                let offset_max = (max_world - image_rect.min) / scale;

                let preview = StrokeRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    stroke_width: drawing_state.stroke_width,
                    stroke_color: drawing_state.stroke_color,
                    rounding: drawing_state.rounding,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::FilledRect => {
                let min_world = egui::pos2(
                    start_world.x.min(end_world.x),
                    start_world.y.min(end_world.y),
                );
                let max_world = egui::pos2(
                    start_world.x.max(end_world.x),
                    start_world.y.max(end_world.y),
                );
                let offset_min = (min_world - image_rect.min) / scale;
                let offset_max = (max_world - image_rect.min) / scale;

                let preview = FilledRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    filled_color: drawing_state.fill_color,
                    rounding: drawing_state.rounding,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::Arrow => {
                let offset_start = (start_world - image_rect.min) / scale;
                let offset_end = (end_world - image_rect.min) / scale;
                let preview = Arrow {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    color: drawing_state.stroke_color,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::Line => {
                let offset_start = (start_world - image_rect.min) / scale;
                let offset_end = (end_world - image_rect.min) / scale;
                let preview = Line {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    stroke_width: drawing_state.stroke_width,
                    stroke_color: drawing_state.stroke_color,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::Mosaic => {
                let min_world = egui::pos2(
                    start_world.x.min(end_world.x),
                    start_world.y.min(end_world.y),
                );
                let max_world = egui::pos2(
                    start_world.x.max(end_world.x),
                    start_world.y.max(end_world.y),
                );
                let offset_min = (min_world - image_rect.min) / scale;
                let offset_max = (max_world - image_rect.min) / scale;

                let preview = Mosaic {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    granularity: drawing_state.mosaic_granularity,
                };
                preview.render(ui, image_rect, scale);
            }
        }
    }
}
