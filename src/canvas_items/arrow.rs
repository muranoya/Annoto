use egui::{
    Pos2, Rect, Ui,
    epaint::{ColorMode, PathShape, PathStroke},
};

#[derive(Clone)]
pub struct Arrow {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,

    pub color: egui::Color32,
}

impl Arrow {
    pub fn render(&self, ui: &mut Ui, image_rect: Rect, scale: f32) {
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

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let line_rad = dy.atan2(dx);

        let path_stroke = PathStroke {
            width: 1.0,
            color: ColorMode::Solid(self.color),
            kind: egui::StrokeKind::Inside,
        };
        let mut points = vec![];
        points.push(Pos2 {
            x: start.x,
            y: start.y,
        });
        points.push(Self::calc_point(&end, line_rad, 20.0, 65.0 * scale));
        points.push(Self::calc_point(&end, line_rad, 30.0, 75.0 * scale));
        points.push(Pos2 { x: end.x, y: end.y });
        points.push(Self::calc_point(&end, line_rad, -30.0, 75.0 * scale));
        points.push(Self::calc_point(&end, line_rad, -20.0, 65.0 * scale));
        let path = PathShape {
            points: points,
            closed: true,
            fill: self.color,
            stroke: path_stroke,
        };
        ui.painter().add(path);
    }

    fn calc_point(p: &Pos2, line_rad: f32, angle_deg: f32, base: f32) -> Pos2 {
        let angle_rad = angle_deg.to_radians();
        let local_x = -base;
        let local_y = base * angle_rad.tan();

        let cos = line_rad.cos();
        let sin = line_rad.sin();

        Pos2 {
            x: p.x + local_x * cos - local_y * sin,
            y: p.y + local_x * sin + local_y * cos,
        }
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
        // Simple line distance check
        let dist = Self::point_to_line_distance(pos, start, end);
        dist < 15.0 // threshold
    }

    pub fn translate(&mut self, delta: egui::Vec2) {
        self.start_x += delta.x;
        self.start_y += delta.y;
        self.end_x += delta.x;
        self.end_y += delta.y;
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

    fn point_to_line_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
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
            self.color.r(),
            self.color.g(),
            self.color.b(),
            self.color.a(),
        );
        paint.anti_alias = true;

        let dx = self.end_x - self.start_x;
        let dy = self.end_y - self.start_y;
        let line_rad = dy.atan2(dx);
        let end_pos = Pos2 {
            x: self.end_x,
            y: self.end_y,
        };

        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(self.start_x, self.start_y);
        let p20 = Self::calc_point(&end_pos, line_rad, 20.0, 65.0);
        path.line_to(p20.x, p20.y);
        let p30 = Self::calc_point(&end_pos, line_rad, 30.0, 75.0);
        path.line_to(p30.x, p30.y);
        path.line_to(self.end_x, self.end_y);
        let pm30 = Self::calc_point(&end_pos, line_rad, -30.0, 75.0);
        path.line_to(pm30.x, pm30.y);
        let pm20 = Self::calc_point(&end_pos, line_rad, -20.0, 65.0);
        path.line_to(pm20.x, pm20.y);
        path.close();
        let path = path.finish().unwrap();

        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
        let stroke_paint = paint.clone();
        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = 1.0;
        pixmap.stroke_path(
            &path,
            &stroke_paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}
