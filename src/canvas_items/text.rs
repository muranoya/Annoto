#[derive(Clone)]
pub struct Text {
    pub x: f32,
    pub y: f32,
    pub content: String,
    pub font_size: f32,
    pub color: egui::Color32,
}

impl Text {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        image_rect: egui::Rect,
        scale: f32,
        selected_index: Option<usize>,
        drawing_mode: bool,
    ) -> Option<usize> {
        let world_pos = image_rect.min
            + (egui::Pos2 {
                x: self.x,
                y: self.y,
            } * scale)
                .to_vec2();

        let mut new_selected = None;
        if !drawing_mode {
            // テキストの境界を計算して選択可能にする
            let galley = ui.painter().layout_no_wrap(
                self.content.clone(),
                egui::FontId::new(self.font_size * scale, egui::FontFamily::Proportional),
                self.color,
            );
            let text_rect = egui::Rect::from_min_size(world_pos, galley.size());

            let response = ui.interact(
                text_rect,
                egui::Id::new(format!("text_{}", index)),
                egui::Sense::click_and_drag(),
            );
            if response.clicked() {
                new_selected = Some(index);
            }
            if response.dragged() {
                let delta = response.drag_delta();
                let delta_scaled = delta / scale;
                self.x += delta_scaled.x;
                self.y += delta_scaled.y;
            }
        }

        let color = if selected_index == Some(index) {
            egui::Color32::YELLOW
        } else {
            self.color
        };

        ui.painter().text(
            world_pos,
            egui::Align2::LEFT_TOP,
            &self.content,
            egui::FontId::new(self.font_size * scale, egui::FontFamily::Proportional),
            color,
        );
        new_selected
    }
}
