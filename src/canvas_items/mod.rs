pub mod arrow;
pub mod filled_rect;
pub mod line;
pub mod mosaic;
pub mod stroke_rect;

pub use arrow::Arrow;
pub use filled_rect::FilledRect;
pub use line::Line;
pub use mosaic::Mosaic;
pub use stroke_rect::StrokeRect;

#[derive(Clone, Debug)]
pub enum Handle {
    Corner(usize),
    Start,
    End,
}

#[derive(Clone)]
pub enum CanvasItem {
    StrokeRect(StrokeRect),
    FilledRect(FilledRect),
    Arrow(Arrow),
    Line(Line),
    Mosaic(Mosaic),
}

impl CanvasItem {
    pub fn hit_test(&self, pos: egui::Pos2, image_rect: egui::Rect, scale: f32) -> bool {
        match self {
            CanvasItem::StrokeRect(item) => item.hit_test(pos, image_rect, scale),
            CanvasItem::FilledRect(item) => item.hit_test(pos, image_rect, scale),
            CanvasItem::Arrow(item) => item.hit_test(pos, image_rect, scale),
            CanvasItem::Line(item) => item.hit_test(pos, image_rect, scale),
            CanvasItem::Mosaic(item) => item.hit_test(pos, image_rect, scale),
        }
    }

    pub fn translate(&mut self, delta: egui::Vec2) {
        match self {
            CanvasItem::StrokeRect(item) => item.translate(delta),
            CanvasItem::FilledRect(item) => item.translate(delta),
            CanvasItem::Arrow(item) => item.translate(delta),
            CanvasItem::Line(item) => item.translate(delta),
            CanvasItem::Mosaic(item) => item.translate(delta),
        }
    }

    pub fn resize(&mut self, handle: &Handle, delta: egui::Vec2) {
        match self {
            CanvasItem::StrokeRect(item) => item.resize(handle, delta),
            CanvasItem::FilledRect(item) => item.resize(handle, delta),
            CanvasItem::Arrow(item) => item.resize(handle, delta),
            CanvasItem::Line(item) => item.resize(handle, delta),
            CanvasItem::Mosaic(item) => item.resize(handle, delta),
        }
    }

    pub fn get_handles(&self, image_rect: egui::Rect, scale: f32) -> Vec<(egui::Pos2, Handle)> {
        match self {
            CanvasItem::StrokeRect(item) => item.get_handles(image_rect, scale),
            CanvasItem::FilledRect(item) => item.get_handles(image_rect, scale),
            CanvasItem::Arrow(item) => item.get_handles(image_rect, scale),
            CanvasItem::Line(item) => item.get_handles(image_rect, scale),
            CanvasItem::Mosaic(item) => item.get_handles(image_rect, scale),
        }
    }

    pub fn get_stroke_width(&self) -> Option<f32> {
        match self {
            CanvasItem::StrokeRect(item) => Some(item.stroke_width),
            CanvasItem::Line(item) => Some(item.stroke_width),
            CanvasItem::Mosaic(item) => Some(item.granularity as f32),
            _ => None,
        }
    }

    pub fn set_stroke_width(&mut self, width: f32) {
        match self {
            CanvasItem::StrokeRect(item) => item.stroke_width = width,
            CanvasItem::Line(item) => item.stroke_width = width,
            CanvasItem::Mosaic(item) => item.set_granularity(width as u8),
            _ => {}
        }
    }

    pub fn get_stroke_color(&self) -> Option<egui::Color32> {
        match self {
            CanvasItem::StrokeRect(item) => Some(item.stroke_color),
            CanvasItem::Arrow(item) => Some(item.color),
            CanvasItem::Line(item) => Some(item.stroke_color),
            _ => None,
        }
    }

    pub fn set_stroke_color(&mut self, color: egui::Color32) {
        match self {
            CanvasItem::StrokeRect(item) => item.stroke_color = color,
            CanvasItem::Arrow(item) => item.color = color,
            CanvasItem::Line(item) => item.stroke_color = color,
            _ => {}
        }
    }

    pub fn get_fill_color(&self) -> Option<egui::Color32> {
        match self {
            CanvasItem::FilledRect(item) => Some(item.filled_color),
            _ => None,
        }
    }

    pub fn set_fill_color(&mut self, color: egui::Color32) {
        match self {
            CanvasItem::FilledRect(item) => item.filled_color = color,
            _ => {}
        }
    }

    pub fn get_rounding(&self) -> Option<u8> {
        match self {
            CanvasItem::StrokeRect(item) => Some(item.rounding),
            CanvasItem::FilledRect(item) => Some(item.rounding),
            _ => None,
        }
    }

    pub fn set_rounding(&mut self, rounding: u8) {
        match self {
            CanvasItem::StrokeRect(item) => item.rounding = rounding,
            CanvasItem::FilledRect(item) => item.rounding = rounding,
            _ => {}
        }
    }

    pub fn scale(&self, factor: f32) -> CanvasItem {
        match self {
            CanvasItem::StrokeRect(item) => CanvasItem::StrokeRect(StrokeRect {
                x1: item.x1 * factor,
                y1: item.y1 * factor,
                x2: item.x2 * factor,
                y2: item.y2 * factor,
                stroke_width: item.stroke_width * factor,
                stroke_color: item.stroke_color,
                rounding: item.rounding,
            }),
            CanvasItem::FilledRect(item) => CanvasItem::FilledRect(FilledRect {
                x1: item.x1 * factor,
                y1: item.y1 * factor,
                x2: item.x2 * factor,
                y2: item.y2 * factor,
                filled_color: item.filled_color,
                rounding: item.rounding,
            }),
            CanvasItem::Arrow(item) => CanvasItem::Arrow(Arrow {
                start_x: item.start_x * factor,
                start_y: item.start_y * factor,
                end_x: item.end_x * factor,
                end_y: item.end_y * factor,
                color: item.color,
            }),
            CanvasItem::Line(item) => CanvasItem::Line(Line {
                start_x: item.start_x * factor,
                start_y: item.start_y * factor,
                end_x: item.end_x * factor,
                end_y: item.end_y * factor,
                stroke_width: item.stroke_width * factor,
                stroke_color: item.stroke_color,
            }),
            CanvasItem::Mosaic(item) => CanvasItem::Mosaic(Mosaic {
                x1: item.x1 * factor,
                y1: item.y1 * factor,
                x2: item.x2 * factor,
                y2: item.y2 * factor,
                granularity: item.granularity,
            }),
        }
    }
}
