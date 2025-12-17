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
    pub fn render(&self, ui: &mut egui::Ui, image_rect: egui::Rect, scale: f32) {
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

        let stroke_width = self.stroke_width * scale;
        ui.painter().rect_stroke(
            world_rect,
            0.0,
            egui::Stroke::new(stroke_width, self.stroke_color),
            egui::StrokeKind::Middle,
        );
    }
}
