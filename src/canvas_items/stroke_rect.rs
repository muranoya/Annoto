#[derive(Clone)]
pub struct StrokeRect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,

    pub stroke_width: f32,
    pub stroke_color: egui::Color32,
    pub rounding: u8,
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
            egui::CornerRadius::same(self.rounding),
            egui::Stroke::new(stroke_width, self.stroke_color),
            egui::StrokeKind::Middle,
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

        let rect =
            tiny_skia::Rect::from_xywh(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1)
                .unwrap();
        let mut path = tiny_skia::PathBuilder::new();
        path.push_rect(rect);
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
