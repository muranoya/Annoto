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

    pub fn hit_test(&self, pos: egui::Pos2, image_rect: egui::Rect, scale: f32) -> bool {
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
        world_rect.contains(pos)
    }

    pub fn translate(&mut self, delta: egui::Vec2) {
        self.x1 += delta.x;
        self.y1 += delta.y;
        self.x2 += delta.x;
        self.y2 += delta.y;
    }

    pub fn resize(&mut self, handle: &crate::canvas_items::Handle, delta: egui::Vec2) {
        match handle {
            crate::canvas_items::Handle::Corner(index) => {
                match *index {
                    0 => {
                        // top-left
                        self.x1 += delta.x;
                        self.y1 += delta.y;
                    }
                    1 => {
                        // top-right
                        self.x2 += delta.x;
                        self.y1 += delta.y;
                    }
                    2 => {
                        // bottom-left
                        self.x1 += delta.x;
                        self.y2 += delta.y;
                    }
                    3 => {
                        // bottom-right
                        self.x2 += delta.x;
                        self.y2 += delta.y;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn get_handles(
        &self,
        image_rect: egui::Rect,
        scale: f32,
    ) -> Vec<(egui::Pos2, crate::canvas_items::Handle)> {
        let mut handles = Vec::new();
        let world_min = image_rect.min
            + egui::Pos2 {
                x: self.x1,
                y: self.y1,
            }
            .to_vec2()
                * scale;
        let world_max = image_rect.min
            + egui::Pos2 {
                x: self.x2,
                y: self.y2,
            }
            .to_vec2()
                * scale;
        handles.push((world_min, crate::canvas_items::Handle::Corner(0)));
        handles.push((
            egui::Pos2::new(world_max.x, world_min.y),
            crate::canvas_items::Handle::Corner(1),
        ));
        handles.push((
            egui::Pos2::new(world_min.x, world_max.y),
            crate::canvas_items::Handle::Corner(2),
        ));
        handles.push((world_max, crate::canvas_items::Handle::Corner(3)));
        handles
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
