use crate::canvas_items::CanvasItem;
use egui;

pub struct ItemRenderer;

impl ItemRenderer {
    /// 既存のアイテムを描画
    pub fn render_existing_items(
        ui: &mut egui::Ui,
        rectangles: &mut [CanvasItem],
        image_rect: egui::Rect,
        scale: f32,
    ) {
        for item in rectangles.iter_mut() {
            match item {
                CanvasItem::StrokeRect(rect) => rect.render(ui, image_rect, scale),
                CanvasItem::FilledRect(rect) => rect.render(ui, image_rect, scale),
                CanvasItem::Arrow(arrow) => arrow.render(ui, image_rect, scale),
                CanvasItem::Line(line) => line.render(ui, image_rect, scale),
                CanvasItem::Mosaic(mosaic) => mosaic.render(ui, image_rect, scale),
            };
        }
    }

    /// ハンドルを描画
    pub fn render_handles(
        ui: &mut egui::Ui,
        selected_item: Option<usize>,
        rectangles: &[CanvasItem],
        image_rect: egui::Rect,
        scale: f32,
        mut on_handle_drag: impl FnMut(usize),
    ) {
        if let Some(selected_idx) = selected_item {
            if let Some(item) = rectangles.get(selected_idx) {
                let handles = item.get_handles(image_rect, scale);
                for (pos, handle) in handles {
                    let rect = egui::Rect::from_center_size(pos, egui::Vec2::splat(10.0));
                    let response = ui.interact(
                        rect,
                        egui::Id::new(format!("handle_{:?}", handle)),
                        egui::Sense::click_and_drag(),
                    );
                    ui.painter().rect_filled(rect, 0.0, egui::Color32::BLUE);
                    if response.dragged() {
                        on_handle_drag(selected_idx);
                    }
                }
            }
        }
    }
}
