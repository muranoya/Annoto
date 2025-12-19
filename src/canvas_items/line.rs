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

    pub fn hit_test(&self, pos: egui::Pos2, image_rect: egui::Rect, scale: f32) -> bool {
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
        let dist = Self::point_to_line_distance(pos, start, end);
        dist < 15.0 // threshold
    }

    pub fn translate(&mut self, delta: egui::Vec2) {
        self.start_x += delta.x;
        self.start_y += delta.y;
        self.end_x += delta.x;
        self.end_y += delta.y;
    }

    pub fn get_handles(
        &self,
        image_rect: egui::Rect,
        scale: f32,
    ) -> Vec<(egui::Pos2, crate::canvas_items::Handle)> {
        let mut handles = Vec::new();
        let start_world = image_rect.min
            + egui::Pos2 {
                x: self.start_x,
                y: self.start_y,
            }
            .to_vec2()
                * scale;
        let end_world = image_rect.min
            + egui::Pos2 {
                x: self.end_x,
                y: self.end_y,
            }
            .to_vec2()
                * scale;
        handles.push((start_world, crate::canvas_items::Handle::Start));
        handles.push((end_world, crate::canvas_items::Handle::End));
        handles
    }

    pub fn resize(&mut self, handle: &crate::canvas_items::Handle, delta: egui::Vec2) {
        match handle {
            crate::canvas_items::Handle::Start => {
                self.start_x += delta.x;
                self.start_y += delta.y;
            }
            crate::canvas_items::Handle::End => {
                self.end_x += delta.x;
                self.end_y += delta.y;
            }
            _ => {}
        }
    }

    fn point_to_line_distance(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
        let ab = b - a;
        let ap = p - a;
        let proj = ap.dot(ab) / ab.length_sq();
        let proj = proj.clamp(0.0, 1.0);
        let closest = a + proj * ab;
        (p - closest).length()
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
