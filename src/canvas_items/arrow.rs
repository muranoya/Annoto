use egui::{
    Pos2,
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

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let line_rad = (dy as f32).atan2(dx as f32);

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
        points.push(Self::calc_point(&end, line_rad, 20.0, 45.0));
        points.push(Self::calc_point(&end, line_rad, 40.0, 55.0));
        points.push(Pos2 { x: end.x, y: end.y });
        points.push(Self::calc_point(&end, line_rad, -40.0, 55.0));
        points.push(Self::calc_point(&end, line_rad, -20.0, 45.0));
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
}
