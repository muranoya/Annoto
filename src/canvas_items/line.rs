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

    pub fn draw_on_pixmap(&self, pixmap: &mut tiny_skia::Pixmap) {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(
            self.stroke_color.r(),
            self.stroke_color.g(),
            self.stroke_color.b(),
            self.stroke_color.a(),
        );
        paint.anti_alias = true;

        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(self.start_x, self.start_y);
        path.line_to(self.end_x, self.end_y);
        let path = path.finish().unwrap();

        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = self.stroke_width;
        pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}
