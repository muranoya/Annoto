#[derive(Clone)]
pub struct Mosaic {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,

    pub granularity: u8, // モザイクの粒度（ピクセル単位）
}

impl Mosaic {
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

        // モザイク領域を半透明の灰色で表示
        ui.painter().rect_filled(
            world_rect,
            0.0,
            egui::Color32::from_rgba_premultiplied(128, 128, 128, 100),
        );

        // 粒度を表示するための枠線
        ui.painter().rect_stroke(
            world_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::GRAY),
            egui::StrokeKind::Middle,
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

    pub fn draw_on_pixmap(&self, pixmap: &mut tiny_skia::Pixmap, _image_bytes: &[u8]) {
        let width = pixmap.width() as usize;
        let height = pixmap.height() as usize;

        let x1 = self.x1.max(0.0) as usize;
        let y1 = self.y1.max(0.0) as usize;
        let x2 = (self.x2.min(width as f32) as usize).min(width);
        let y2 = (self.y2.min(height as f32) as usize).min(height);

        let granularity = self.granularity.max(1) as usize;

        // モザイク処理
        for by in (y1..y2).step_by(granularity) {
            for bx in (x1..x2).step_by(granularity) {
                // ブロック内の平均色を計算
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;

                for py in by..((by + granularity).min(y2)) {
                    for px in bx..((bx + granularity).min(x2)) {
                        let idx = py * width + px;
                        if idx < pixmap.pixels().len() {
                            let pixel = pixmap.pixels()[idx];
                            r_sum += pixel.red() as u32;
                            g_sum += pixel.green() as u32;
                            b_sum += pixel.blue() as u32;
                            a_sum += pixel.alpha() as u32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let avg_r = (r_sum / count) as u8;
                    let avg_g = (g_sum / count) as u8;
                    let avg_b = (b_sum / count) as u8;
                    let avg_a = (a_sum / count) as u8;

                    // ブロック内のすべてのピクセルを平均色で塗りつぶし
                    for py in by..((by + granularity).min(y2)) {
                        for px in bx..((bx + granularity).min(x2)) {
                            let idx = py * width + px;
                            if idx < pixmap.pixels_mut().len() {
                                let color =
                                    tiny_skia::Color::from_rgba8(avg_r, avg_g, avg_b, avg_a);
                                pixmap.pixels_mut()[idx] = color.premultiply().to_color_u8();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_granularity(&self) -> u8 {
        self.granularity
    }

    pub fn set_granularity(&mut self, granularity: u8) {
        self.granularity = granularity.max(1);
    }
}
