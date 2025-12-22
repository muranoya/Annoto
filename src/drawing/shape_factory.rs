use crate::canvas_items::*;
use crate::drawing_tool::DrawingTool;
use crate::state::DrawingState;
use egui;

pub struct ShapeFactory;

impl ShapeFactory {
    /// ドラッグ開始と終了の座標から図形を生成
    pub fn create_shape_from_drag(
        tool: DrawingTool,
        start: egui::Pos2,
        end: egui::Pos2,
        image_rect: egui::Rect,
        scale: f32,
        drawing_state: &DrawingState,
    ) -> Option<CanvasItem> {
        match tool {
            DrawingTool::StrokeRect => {
                let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                let offset_min = (min - image_rect.min) / scale;
                let offset_max = (max - image_rect.min) / scale;
                Some(CanvasItem::StrokeRect(StrokeRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    stroke_width: drawing_state.stroke_width,
                    stroke_color: drawing_state.stroke_color,
                    rounding: drawing_state.rounding,
                }))
            }
            DrawingTool::FilledRect => {
                let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                let offset_min = (min - image_rect.min) / scale;
                let offset_max = (max - image_rect.min) / scale;
                Some(CanvasItem::FilledRect(FilledRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    filled_color: drawing_state.fill_color,
                    rounding: drawing_state.rounding,
                }))
            }
            DrawingTool::Arrow => {
                let offset_start = (start - image_rect.min) / scale;
                let offset_end = (end - image_rect.min) / scale;
                Some(CanvasItem::Arrow(Arrow {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    color: drawing_state.stroke_color,
                }))
            }
            DrawingTool::Line => {
                let offset_start = (start - image_rect.min) / scale;
                let offset_end = (end - image_rect.min) / scale;
                Some(CanvasItem::Line(Line {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    stroke_width: drawing_state.stroke_width,
                    stroke_color: drawing_state.stroke_color,
                }))
            }
            DrawingTool::Mosaic => {
                let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                let offset_min = (min - image_rect.min) / scale;
                let offset_max = (max - image_rect.min) / scale;
                Some(CanvasItem::Mosaic(Mosaic {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    granularity: drawing_state.mosaic_granularity,
                }))
            }
        }
    }
}
