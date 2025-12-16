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
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        image_rect: egui::Rect,
        scale: f32,
        selected_index: Option<usize>,
        drawing_mode: bool,
    ) -> Option<usize> {
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

        let mut new_selected = None;
        if !drawing_mode {
            // 線の周囲にヒットボックスを作成（ドラッグ用）
            let thickness = self.stroke_width * scale;
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0 {
                let unit_dx = dx / length;
                let unit_dy = dy / length;
                let perp_dx = -unit_dy * thickness / 2.0;
                let perp_dy = unit_dx * thickness / 2.0;

                let points = vec![
                    egui::Pos2 {
                        x: start.x + perp_dx,
                        y: start.y + perp_dy,
                    },
                    egui::Pos2 {
                        x: start.x - perp_dx,
                        y: start.y - perp_dy,
                    },
                    egui::Pos2 {
                        x: end.x - perp_dx,
                        y: end.y - perp_dy,
                    },
                    egui::Pos2 {
                        x: end.x + perp_dx,
                        y: end.y + perp_dy,
                    },
                ];
                let bounding_rect = egui::Rect::from_points(&points);

                let response = ui.interact(
                    bounding_rect,
                    egui::Id::new(format!("line_{}", index)),
                    egui::Sense::click_and_drag(),
                );
                if response.clicked() {
                    new_selected = Some(index);
                }
                if response.dragged() {
                    let delta = response.drag_delta();
                    let delta_scaled = delta / scale;
                    self.start_x += delta_scaled.x;
                    self.start_y += delta_scaled.y;
                    self.end_x += delta_scaled.x;
                    self.end_y += delta_scaled.y;
                }
            }
        }

        let color = if selected_index == Some(index) {
            egui::Color32::YELLOW
        } else {
            self.stroke_color
        };
        let stroke_width = self.stroke_width * scale;
        ui.painter()
            .line_segment([start, end], egui::Stroke::new(stroke_width, color));
        new_selected
    }
}
