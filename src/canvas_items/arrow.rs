#[derive(Clone)]
pub struct Arrow {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub color: egui::Color32,
    pub width: f32,
}

impl Arrow {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        image_rect: egui::Rect,
        scale: f32,
        selected_index: Option<usize>,
        drawing_mode: bool,
        changed: &mut bool,
    ) -> Option<usize> {
        let start = image_rect.min
            + (egui::Pos2 {
                x: self.start_x,
                y: self.start_y,
            } * scale)
                .to_vec2();
        let end = image_rect.min
            + (egui::Pos2 {
                x: self.end_x,
                y: self.end_y,
            } * scale)
                .to_vec2();
        let color = if selected_index == Some(index) {
            egui::Color32::YELLOW
        } else {
            self.color
        };
        ui.painter()
            .line_segment([start, end], egui::Stroke::new(self.width * scale, color));
        // 矢印の頭
        let dir = (end - start).normalized();
        let arrow_size = 10.0 * scale;
        let perp = egui::Vec2::new(-dir.y, dir.x);
        let left = end - dir * arrow_size + perp * arrow_size * 0.5;
        let right = end - dir * arrow_size - perp * arrow_size * 0.5;
        ui.painter()
            .line_segment([end, left], egui::Stroke::new(self.width * scale, color));
        ui.painter()
            .line_segment([end, right], egui::Stroke::new(self.width * scale, color));
        let rect = egui::Rect::from_min_max(start.min(end), end).expand(5.0);
        let mut new_selected = None;
        if !drawing_mode {
            let response = ui.interact(
                rect,
                egui::Id::new(format!("arrow_{}", index)),
                egui::Sense::click_and_drag(),
            );
            if response.clicked() {
                new_selected = Some(index);
            }
            if response.dragged() {
                let delta = response.drag_delta() / scale;
                self.start_x += delta.x;
                self.start_y += delta.y;
                self.end_x += delta.x;
                self.end_y += delta.y;
                *changed = true;
            }
        }
        new_selected
    }
}
