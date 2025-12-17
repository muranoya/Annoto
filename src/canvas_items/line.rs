#[derive(Clone)]
pub struct Line {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,

    pub stroke_width: f32,
    pub stroke_color: egui::Color32,
}

impl Line {
    pub fn render(&self, ui: &mut egui::Ui, image_rect: egui::Rect, scale: f32) {
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

        let stroke_width = self.stroke_width * scale;
        ui.painter().line_segment(
            [start, end],
            egui::Stroke::new(stroke_width, self.stroke_color),
        );
    }
}
