use egui::{FontData, FontDefinitions, FontFamily};
use js_sys;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{File, FileReader, HtmlInputElement};

struct AppState {
    image_bytes: Option<Vec<u8>>,
}

static APP_STATE: Lazy<Arc<Mutex<AppState>>> =
    Lazy::new(|| Arc::new(Mutex::new(AppState { image_bytes: None })));

enum CanvasItem {
    StrokeRect(StrokeRect),
    FilledRect(FilledRect),
}

struct FilledRect {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,

    filled_color: egui::Color32,
}

struct StrokeRect {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,

    stroke_width: f32,
    stroke_color: egui::Color32,
}

pub struct AnnotoApp {
    image_texture: Option<egui::TextureHandle>,
    image_bytes: Option<Vec<u8>>,

    zoom: f32,
    drawing_mode: bool,
    rectangles: Vec<CanvasItem>,
    drag_start: Option<egui::Pos2>,
    selected_index: Option<usize>,
    stroke_width: f32,
    stroke_color: egui::Color32,
    fill_enabled: bool,
    fill_color: egui::Color32,
}

impl Default for AnnotoApp {
    fn default() -> Self {
        Self {
            image_texture: None,
            image_bytes: None,
            zoom: 100.0,
            drawing_mode: false,
            rectangles: Vec::new(),
            drag_start: None,
            selected_index: None,
            stroke_width: 3.0,
            stroke_color: egui::Color32::RED,
            fill_enabled: false,
            fill_color: egui::Color32::from_rgba_premultiplied(255, 0, 0, 128),
        }
    }
}

impl AnnotoApp {
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "NotoSansRegular".to_owned(),
            std::sync::Arc::new(FontData::from_static(include_bytes!(
                "../fonts/NotoSansJP-Regular.ttf"
            ))),
        );
        // Put my font first (highest priority):
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "NotoSansRegular".to_owned());
        // Put my font as last fallback for monospace:
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("my_font".to_owned());
        ctx.set_fonts(fonts);
    }

    fn open_file_dialog() {
        let document = web_sys::window().unwrap().document().unwrap();
        let input = document.create_element("input").unwrap();
        let input: HtmlInputElement = input.dyn_into().unwrap();
        input.set_type("file");
        input.set_accept("image/*");
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    Self::load_image(file);
                }
            }
        }) as Box<dyn FnMut(_)>);
        input
            .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
        input.click();
    }

    fn load_image(file: File) {
        let reader = FileReader::new().unwrap();
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let reader: FileReader = event.target().unwrap().dyn_into().unwrap();
            if let Ok(result) = reader.result() {
                let array_buffer = js_sys::ArrayBuffer::from(result);
                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                let bytes = uint8_array.to_vec();
                APP_STATE.lock().unwrap().image_bytes = Some(bytes);
            }
        }) as Box<dyn FnMut(_)>);
        reader
            .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
        reader.read_as_array_buffer(&file).unwrap();
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        Default::default()
    }
}

impl eframe::App for AnnotoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(bytes) = APP_STATE.lock().unwrap().image_bytes.take() {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.image_texture =
                    Some(ctx.load_texture("image", color_image, egui::TextureOptions::default()));
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("ファイルを開く").clicked() {
                        Self::open_file_dialog();
                    }
                });
                ui.add_space(16.0);
                ui.label("倍率:");
                ui.add(
                    egui::DragValue::new(&mut self.zoom)
                        .range(1.0..=500.0)
                        .suffix("%"),
                );
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label(if self.drawing_mode {
                "描画モード"
            } else {
                "選択モード"
            });
            if ui
                .button(if self.drawing_mode {
                    "選択モードに切替"
                } else {
                    "描画モードに切替"
                })
                .clicked()
            {
                self.drawing_mode = !self.drawing_mode;
            }
            ui.add_space(16.0);
            ui.label("線の太さ:");
            ui.add(
                egui::DragValue::new(&mut self.stroke_width)
                    .range(1..=50)
                    .suffix("px"),
            );
            ui.add_space(16.0);
            ui.label("線の色:");
            ui.color_edit_button_srgba(&mut self.stroke_color);
            ui.add_space(16.0);
            ui.checkbox(&mut self.fill_enabled, "塗りつぶし");
            if self.fill_enabled {
                ui.color_edit_button_srgba(&mut self.fill_color);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.image_texture {
                let scale = self.zoom / 100.0;
                let scaled_size = texture.size_vec2() * scale;
                let image_response =
                    ui.allocate_response(scaled_size, egui::Sense::click_and_drag());
                let image_rect = image_response.rect;

                // 画像を描画
                ui.painter().image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                    egui::Color32::WHITE,
                );

                // 描画モード時の処理
                if self.drawing_mode {
                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                    if let Some(pos) = pointer_pos {
                        if image_rect.contains(pos) {
                            if image_response.drag_started() {
                                self.drag_start = Some(pos);
                            }
                            if image_response.drag_stopped() {
                                if let Some(start) = self.drag_start {
                                    let end = pos;
                                    let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                                    let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                                    let offset_min = (min - image_rect.min) / scale;
                                    let offset_max = (max - image_rect.min) / scale;
                                    if self.fill_enabled {
                                        self.rectangles.push(CanvasItem::FilledRect(FilledRect {
                                            x1: offset_min.x,
                                            y1: offset_min.y,
                                            x2: offset_max.x,
                                            y2: offset_max.y,
                                            filled_color: self.fill_color,
                                        }));
                                    } else {
                                        self.rectangles.push(CanvasItem::StrokeRect(StrokeRect {
                                            x1: offset_min.x,
                                            y1: offset_min.y,
                                            x2: offset_max.x,
                                            y2: offset_max.y,
                                            stroke_width: self.stroke_width,
                                            stroke_color: self.stroke_color,
                                        }));
                                    }
                                    self.drag_start = None;
                                }
                            }
                        }
                    }
                }

                // 既存の四角形を描画
                for (index, item) in self.rectangles.iter_mut().enumerate() {
                    match item {
                        CanvasItem::StrokeRect(rect) => {
                            let world_min = image_rect.min
                                + (egui::Pos2 {
                                    x: rect.x1,
                                    y: rect.y1,
                                } * scale)
                                    .to_vec2();
                            let world_max = image_rect.min
                                + (egui::Pos2 {
                                    x: rect.x2,
                                    y: rect.y2,
                                } * scale)
                                    .to_vec2();
                            let world_rect = egui::Rect::from_min_max(world_min, world_max);

                            if !self.drawing_mode {
                                let response = ui.interact(
                                    world_rect,
                                    egui::Id::new(format!("rect_{}", index)),
                                    egui::Sense::click_and_drag(),
                                );
                                if response.clicked() {
                                    self.selected_index = Some(index);
                                }
                                if response.dragged() {
                                    let delta = response.drag_delta();
                                    let delta_scaled = delta / scale;
                                    rect.x1 += delta_scaled.x;
                                    rect.y1 += delta_scaled.y;
                                    rect.x2 += delta_scaled.x;
                                    rect.y2 += delta_scaled.y;
                                }
                            }

                            let color = if self.selected_index == Some(index) {
                                egui::Color32::YELLOW
                            } else {
                                rect.stroke_color
                            };
                            let stroke_width = rect.stroke_width * scale;
                            ui.painter().rect_stroke(
                                world_rect,
                                0.0,
                                egui::Stroke::new(stroke_width, color),
                                egui::StrokeKind::Middle,
                            );
                        }
                        CanvasItem::FilledRect(rect) => {
                            let world_min = image_rect.min
                                + (egui::Pos2 {
                                    x: rect.x1,
                                    y: rect.y1,
                                } * scale)
                                    .to_vec2();
                            let world_max = image_rect.min
                                + (egui::Pos2 {
                                    x: rect.x2,
                                    y: rect.y2,
                                } * scale)
                                    .to_vec2();
                            let world_rect = egui::Rect::from_min_max(world_min, world_max);

                            if !self.drawing_mode {
                                let response = ui.interact(
                                    world_rect,
                                    egui::Id::new(format!("rect_{}", index)),
                                    egui::Sense::click_and_drag(),
                                );
                                if response.clicked() {
                                    self.selected_index = Some(index);
                                }
                                if response.dragged() {
                                    let delta = response.drag_delta();
                                    let delta_scaled = delta / scale;
                                    rect.x1 += delta_scaled.x;
                                    rect.y1 += delta_scaled.y;
                                    rect.x2 += delta_scaled.x;
                                    rect.y2 += delta_scaled.y;
                                }
                            }

                            ui.painter().rect_filled(world_rect, 0.0, rect.filled_color);
                        }
                    }
                }

                // ドラッグ中の四角形を描画
                if let Some(start) = self.drag_start {
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        if image_rect.contains(pos) {
                            let end = pos;
                            let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                            let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                            let world_rect = egui::Rect::from_min_max(min, max);
                            if self.fill_enabled {
                                ui.painter().rect_filled(world_rect, 0.0, self.fill_color);
                            }
                            let stroke_width = self.stroke_width * scale;
                            ui.painter().rect_stroke(
                                world_rect,
                                0.0,
                                egui::Stroke::new(stroke_width, egui::Color32::BLUE),
                                egui::StrokeKind::Middle,
                            );
                        }
                    }
                }
            }
        });
    }
}
