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
    ) -> bool {
        let mut should_delete = false;
        if let Some(selected_idx) = selected_item {
            // ハンドル情報を先に取得
            let (handles, is_rect) = if let Some(item) = rectangles.get(selected_idx) {
                let handles = item.get_handles(image_rect, scale);
                let is_rect = matches!(
                    item,
                    CanvasItem::StrokeRect(_) | CanvasItem::FilledRect(_) | CanvasItem::Mosaic(_)
                );
                (handles, is_rect)
            } else {
                (Vec::new(), false)
            };

            let mut max_pos = egui::Pos2::ZERO;
            let mut delete_handle_pos = egui::Pos2::ZERO;

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

                // 最大座標を追跡
                if pos.x > max_pos.x {
                    max_pos.x = pos.x;
                }
                if pos.y > max_pos.y {
                    max_pos.y = pos.y;
                }

                // 四角形の場合は右上のハンドル（インデックス1）を削除ハンドルの基準位置として使用
                if is_rect && i == 1 {
                    delete_handle_pos = pos;
                }
            }

            // 削除ハンドルを描画
            // 図形の種類に応じて位置を調整
            let delete_pos = if is_rect {
                // 四角形: 右上のハンドルの外側に10px離した位置
                delete_handle_pos + egui::Vec2::new(15.0, -15.0)
            } else {
                // 矢印および線: 最大座標からもう少し離した位置
                max_pos + egui::Vec2::new(15.0, -15.0)
            };
            let delete_rect = egui::Rect::from_center_size(delete_pos, egui::Vec2::splat(20.0));
            let response = ui.interact(
                delete_rect,
                egui::Id::new(format!("delete_handle_{}", selected_idx)),
                egui::Sense::click(),
            );
            ui.painter()
                .rect_filled(delete_rect, 2.0, egui::Color32::RED);
            ui.painter().text(
                delete_pos,
                egui::Align2::CENTER_CENTER,
                "×",
                egui::FontId::default(),
                egui::Color32::WHITE,
            );
            if response.clicked() {
                should_delete = true;
            }
        }
        should_delete
    }
}
