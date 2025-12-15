pub mod arrow;
pub mod filled_rect;
pub mod stroke_rect;

pub use arrow::Arrow;
pub use filled_rect::FilledRect;
pub use stroke_rect::StrokeRect;

#[derive(Clone)]
pub enum CanvasItem {
    StrokeRect(StrokeRect),
    FilledRect(FilledRect),
    Arrow(Arrow),
}
