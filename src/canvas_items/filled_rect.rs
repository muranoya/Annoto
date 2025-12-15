#[derive(Clone)]
pub struct FilledRect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,

    pub filled_color: egui::Color32,
}

impl FilledRect {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        image_rect: egui::Rect,
        scale: f32,
        _selected_index: Option<usize>,
        drawing_mode: bool,
    ) -> Option<usize> {
        let world_min = image_rect.min
            + (egui::Pos2 {
                x: self.x1,
                y: self.y1,
            } * scale)
                .to_vec2();
        let world_max = image_rect.min
            + (egui::Pos2 {
                x: self.x2,
                y: self.y2,
            } * scale)
                .to_vec2();
        let world_rect = egui::Rect::from_min_max(world_min, world_max);

        let mut new_selected = None;
        if !drawing_mode {
            let response = ui.interact(
                world_rect,
                egui::Id::new(format!("rect_{}", index)),
                egui::Sense::click_and_drag(),
            );
            if response.clicked() {
                new_selected = Some(index);
            }
            if response.dragged() {
                let delta = response.drag_delta();
                let delta_scaled = delta / scale;
                self.x1 += delta_scaled.x;
                self.y1 += delta_scaled.y;
                self.x2 += delta_scaled.x;
                self.y2 += delta_scaled.y;
            }
        }

        ui.painter().rect_filled(world_rect, 0.0, self.filled_color);
        new_selected
    }
}
