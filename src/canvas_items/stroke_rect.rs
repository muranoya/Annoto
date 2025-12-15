#[derive(Clone)]
pub struct StrokeRect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,

    pub stroke_width: f32,
    pub stroke_color: egui::Color32,
}

impl StrokeRect {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        image_rect: egui::Rect,
        scale: f32,
        selected_index: Option<usize>,
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

        let color = if selected_index == Some(index) {
            egui::Color32::YELLOW
        } else {
            self.stroke_color
        };
        let stroke_width = self.stroke_width * scale;
        ui.painter().rect_stroke(
            world_rect,
            0.0,
            egui::Stroke::new(stroke_width, color),
            egui::StrokeKind::Middle,
        );
        new_selected
    }
}
