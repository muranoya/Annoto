#[derive(Clone)]
pub struct FilledRect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,

    pub filled_color: egui::Color32,
    pub rounding: u8,
}

impl FilledRect {
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

        ui.painter().rect_filled(
            world_rect,
            egui::CornerRadius::same(self.rounding),
            self.filled_color,
        );
    }

    pub fn draw_on_pixmap(&self, pixmap: &mut tiny_skia::Pixmap) {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(
            self.filled_color.r(),
            self.filled_color.g(),
            self.filled_color.b(),
            self.filled_color.a(),
        );
        paint.anti_alias = true;

        let rect =
            tiny_skia::Rect::from_xywh(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1)
                .unwrap();
        let mut path = tiny_skia::PathBuilder::new();
        path.push_rect(rect);
        let path = path.finish().unwrap();

        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}
