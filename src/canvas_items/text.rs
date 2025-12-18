#[derive(Clone)]
pub struct Text {
    pub x: f32,
    pub y: f32,
    pub content: String,
    pub font_size: f32,
    pub color: egui::Color32,
}

impl Text {
    pub fn render(&self, ui: &mut egui::Ui, image_rect: egui::Rect, scale: f32) {
        let world_pos = image_rect.min
            + (egui::Pos2 {
                x: self.x,
                y: self.y,
            } * scale)
                .to_vec2();

        ui.painter().text(
            world_pos,
            egui::Align2::LEFT_TOP,
            &self.content,
            egui::FontId::new(self.font_size * scale, egui::FontFamily::Proportional),
            self.color,
        );
    }
}
