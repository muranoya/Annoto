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
        selected_handle: &mut Option<crate::canvas_items::Handle>,
        rectangles: &mut [CanvasItem],
        image_rect: egui::Rect,
        scale: f32,
    ) {
        if let Some(selected_idx) = selected_item {
            if let Some(item) = rectangles.get(selected_idx) {
                let handles = item.get_handles(image_rect, scale);
                for (i, (pos, handle)) in handles.into_iter().enumerate() {
                    let rect = egui::Rect::from_center_size(pos, egui::Vec2::splat(10.0));
                    let response = ui.interact(
                        rect,
                        egui::Id::new(format!("handle_{}_{}_{:?}", selected_idx, i, handle)),
                        egui::Sense::click_and_drag(),
                    );
                    ui.painter().rect_filled(rect, 0.0, egui::Color32::BLUE);
                    if response.clicked() {
                        *selected_handle = Some(handle.clone());
                    }
                    if response.dragged() {
                        let delta = response.drag_delta() / scale;
                        if let Some(item_mut) = rectangles.get_mut(selected_idx) {
                            item_mut.resize(&handle, delta);
                        }
                    }
                }
            }
        }
    }
}
