use crate::canvas_items::*;
use crate::drawing_tool::DrawingTool;
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

pub struct AnnotoApp {
    image_texture: Option<egui::TextureHandle>,
    image_bytes: Option<Vec<u8>>,

    zoom: f32,
    rectangles: Vec<CanvasItem>,
    drag_start: Option<egui::Pos2>,
    stroke_width: f32,
    stroke_color: egui::Color32,
    fill_color: egui::Color32,
    rounding: u8,
    current_tool: DrawingTool,

    // Cursor position
    cursor_pos: Option<egui::Pos2>,

    // Export related
    show_export_dialog: bool,
    export_format: String,
}

impl Default for AnnotoApp {
    fn default() -> Self {
        Self {
            image_texture: None,
            image_bytes: None,
            zoom: 100.0,
            rectangles: Vec::new(),
            drag_start: None,
            stroke_width: 3.0,
            stroke_color: egui::Color32::RED,
            fill_color: egui::Color32::from_rgba_premultiplied(255, 0, 0, 128),
            rounding: 0,
            current_tool: DrawingTool::StrokeRect,
            cursor_pos: None,
            show_export_dialog: false,
            export_format: "PNG".to_string(),
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
        self.handle_image_loading(ctx);
        self.render_top_panel(ctx);
        self.render_side_panel(ctx);
        self.render_central_panel(ctx);
        self.show_export_dialog(ctx);
    }
}

impl AnnotoApp {
    fn handle_image_loading(&mut self, ctx: &egui::Context) {
        if let Some(bytes) = APP_STATE.lock().unwrap().image_bytes.take() {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.image_texture =
                    Some(ctx.load_texture("image", color_image, egui::TextureOptions::default()));
                self.image_bytes = Some(bytes);
            }
        }
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("ファイルを開く").clicked() {
                        Self::open_file_dialog();
                    }
                    if ui.button("エクスポート").clicked() {
                        self.show_export_dialog = true;
                    }
                });
                ui.add_space(16.0);
                ui.label("倍率:");
                ui.add(
                    egui::DragValue::new(&mut self.zoom)
                        .range(1.0..=500.0)
                        .suffix("%"),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(pos) = self.cursor_pos {
                        ui.label(format!("X: {:.0}, Y: {:.0}", pos.x, pos.y));
                    }
                });
            });
        });
    }

    fn render_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("描画ツール");
            if ui
                .selectable_label(
                    matches!(self.current_tool, DrawingTool::StrokeRect),
                    "四角形",
                )
                .clicked()
            {
                self.current_tool = DrawingTool::StrokeRect;
            }
            if ui
                .selectable_label(
                    matches!(self.current_tool, DrawingTool::FilledRect),
                    "塗りつぶし四角形",
                )
                .clicked()
            {
                self.current_tool = DrawingTool::FilledRect;
            }
            if ui
                .selectable_label(matches!(self.current_tool, DrawingTool::Arrow), "矢印")
                .clicked()
            {
                self.current_tool = DrawingTool::Arrow;
            }
            if ui
                .selectable_label(matches!(self.current_tool, DrawingTool::Line), "直線")
                .clicked()
            {
                self.current_tool = DrawingTool::Line;
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
            if matches!(self.current_tool, DrawingTool::FilledRect) {
                ui.add_space(16.0);
                ui.label("塗りつぶし色:");
                ui.color_edit_button_srgba(&mut self.fill_color);
            }
            if matches!(
                self.current_tool,
                DrawingTool::StrokeRect | DrawingTool::FilledRect
            ) {
                ui.add_space(16.0);
                ui.label("角の丸め:");
                ui.add(
                    egui::DragValue::new(&mut self.rounding)
                        .range(0..=255)
                        .suffix("px"),
                );
            }
        });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.image_texture.clone() {
                let scale = self.zoom / 100.0;

                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // Update cursor position
                        let pointer_pos = ui.input(|i| i.pointer.hover_pos());

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

                        // Adjust cursor position if over image
                        if let Some(pos) = pointer_pos {
                            if image_rect.contains(pos) {
                                let cursor_in_image = (pos - image_rect.min) / scale;
                                self.cursor_pos =
                                    Some(egui::Pos2::new(cursor_in_image.x, cursor_in_image.y));
                            }
                        } else {
                            self.cursor_pos = None;
                        }

                        self.handle_drawing_mode(ui, &image_response, image_rect, scale);
                        self.render_existing_items(ui, image_rect, scale);
                        self.render_drag_preview(ui, image_rect, scale);
                    });
            }
        });
    }

    fn handle_drawing_mode(
        &mut self,
        ui: &mut egui::Ui,
        image_response: &egui::Response,
        image_rect: egui::Rect,
        scale: f32,
    ) {
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        if let Some(pos) = pointer_pos {
            if image_rect.contains(pos) {
                match self.current_tool {
                    _ => {
                        if image_response.drag_started() {
                            self.drag_start = Some(pos);
                        }
                        if image_response.drag_stopped() {
                            if let Some(start) = self.drag_start {
                                let end = pos;
                                match self.current_tool {
                                    DrawingTool::StrokeRect => {
                                        let min =
                                            egui::pos2(start.x.min(end.x), start.y.min(end.y));
                                        let max =
                                            egui::pos2(start.x.max(end.x), start.y.max(end.y));
                                        let offset_min = (min - image_rect.min) / scale;
                                        let offset_max = (max - image_rect.min) / scale;
                                        self.rectangles.push(CanvasItem::StrokeRect(StrokeRect {
                                            x1: offset_min.x,
                                            y1: offset_min.y,
                                            x2: offset_max.x,
                                            y2: offset_max.y,
                                            stroke_width: self.stroke_width,
                                            stroke_color: self.stroke_color,
                                            rounding: self.rounding,
                                        }));
                                    }
                                    DrawingTool::FilledRect => {
                                        let min =
                                            egui::pos2(start.x.min(end.x), start.y.min(end.y));
                                        let max =
                                            egui::pos2(start.x.max(end.x), start.y.max(end.y));
                                        let offset_min = (min - image_rect.min) / scale;
                                        let offset_max = (max - image_rect.min) / scale;
                                        self.rectangles.push(CanvasItem::FilledRect(FilledRect {
                                            x1: offset_min.x,
                                            y1: offset_min.y,
                                            x2: offset_max.x,
                                            y2: offset_max.y,
                                            filled_color: self.fill_color,
                                            rounding: self.rounding,
                                        }));
                                    }
                                    DrawingTool::Arrow => {
                                        let offset_start = (start - image_rect.min) / scale;
                                        let offset_end = (end - image_rect.min) / scale;
                                        self.rectangles.push(CanvasItem::Arrow(Arrow {
                                            start_x: offset_start.x,
                                            start_y: offset_start.y,
                                            end_x: offset_end.x,
                                            end_y: offset_end.y,
                                            color: self.stroke_color,
                                        }));
                                    }
                                    DrawingTool::Line => {
                                        let offset_start = (start - image_rect.min) / scale;
                                        let offset_end = (end - image_rect.min) / scale;
                                        self.rectangles.push(CanvasItem::Line(Line {
                                            start_x: offset_start.x,
                                            start_y: offset_start.y,
                                            end_x: offset_end.x,
                                            end_y: offset_end.y,
                                            stroke_width: self.stroke_width,
                                            stroke_color: self.stroke_color,
                                        }));
                                    }
                                }
                                self.drag_start = None;
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_existing_items(&mut self, ui: &mut egui::Ui, image_rect: egui::Rect, scale: f32) {
        for item in self.rectangles.iter_mut() {
            match item {
                CanvasItem::StrokeRect(rect) => rect.render(ui, image_rect, scale),
                CanvasItem::FilledRect(rect) => rect.render(ui, image_rect, scale),
                CanvasItem::Arrow(arrow) => arrow.render(ui, image_rect, scale),
                CanvasItem::Line(line) => line.render(ui, image_rect, scale),
            };
        }
    }

    fn render_drag_preview(&mut self, ui: &mut egui::Ui, image_rect: egui::Rect, scale: f32) {
        let Some(start_world) = self.drag_start else {
            return;
        };
        let Some(end_world) = ui.input(|i| i.pointer.hover_pos()) else {
            return;
        };
        if !image_rect.contains(end_world) {
            return;
        }

        match self.current_tool {
            DrawingTool::StrokeRect => {
                let min_world = egui::pos2(
                    start_world.x.min(end_world.x),
                    start_world.y.min(end_world.y),
                );
                let max_world = egui::pos2(
                    start_world.x.max(end_world.x),
                    start_world.y.max(end_world.y),
                );
                let offset_min = (min_world - image_rect.min) / scale;
                let offset_max = (max_world - image_rect.min) / scale;

                let preview = StrokeRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    stroke_width: self.stroke_width,
                    stroke_color: self.stroke_color,
                    rounding: self.rounding,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::FilledRect => {
                let min_world = egui::pos2(
                    start_world.x.min(end_world.x),
                    start_world.y.min(end_world.y),
                );
                let max_world = egui::pos2(
                    start_world.x.max(end_world.x),
                    start_world.y.max(end_world.y),
                );
                let offset_min = (min_world - image_rect.min) / scale;
                let offset_max = (max_world - image_rect.min) / scale;

                let preview = FilledRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    filled_color: self.fill_color,
                    rounding: self.rounding,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::Arrow => {
                let offset_start = (start_world - image_rect.min) / scale;
                let offset_end = (end_world - image_rect.min) / scale;
                let preview = Arrow {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    color: self.stroke_color,
                };
                preview.render(ui, image_rect, scale);
            }
            DrawingTool::Line => {
                let offset_start = (start_world - image_rect.min) / scale;
                let offset_end = (end_world - image_rect.min) / scale;
                let preview = Line {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    stroke_width: self.stroke_width,
                    stroke_color: self.stroke_color,
                };
                preview.render(ui, image_rect, scale);
            }
        }
    }

    fn show_export_dialog(&mut self, ctx: &egui::Context) {
        if self.show_export_dialog {
            let mut open = true;
            egui::Window::new("エクスポート")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("出力フォーマット:");
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.export_format == "PNG", "PNG")
                            .clicked()
                        {
                            self.export_format = "PNG".to_string();
                        }
                        if ui
                            .selectable_label(self.export_format == "JPEG", "JPEG")
                            .clicked()
                        {
                            self.export_format = "JPEG".to_string();
                        }
                    });
                    ui.horizontal(|ui| {
                        let can_export = self.image_bytes.is_some();
                        if ui
                            .add_enabled(can_export, egui::Button::new("エクスポート"))
                            .clicked()
                        {
                            self.export_image();
                            self.show_export_dialog = false;
                        }
                        if ui.button("キャンセル").clicked() {
                            self.show_export_dialog = false;
                        }
                    });
                    if self.image_bytes.is_none() {
                        ui.label("画像をロードしてください。");
                    }
                });
            if !open {
                self.show_export_dialog = false;
            }
        }
    }

    fn export_image(&self) {
        web_sys::console::log_1(&"Exporting image".into());
        if let Some(image_bytes) = &self.image_bytes {
            if let Ok(img) = image::load_from_memory(image_bytes) {
                let rgba_img = img.to_rgba8();
                let width = rgba_img.width() as u32;
                let height = rgba_img.height() as u32;
                let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
                // Copy image data
                for (i, pixel) in rgba_img.pixels().enumerate() {
                    let color =
                        tiny_skia::Color::from_rgba8(pixel[0], pixel[1], pixel[2], pixel[3]);
                    pixmap.pixels_mut()[i] = color.premultiply().to_color_u8();
                }
                // Draw shapes on the pixmap
                for item in &self.rectangles {
                    match item {
                        CanvasItem::StrokeRect(rect) => rect.draw_on_pixmap(&mut pixmap),
                        CanvasItem::FilledRect(rect) => rect.draw_on_pixmap(&mut pixmap),
                        CanvasItem::Arrow(arrow) => arrow.draw_on_pixmap(&mut pixmap),
                        CanvasItem::Line(line) => line.draw_on_pixmap(&mut pixmap),
                    }
                }
                // Convert pixmap to RgbaImage
                let mut rgba_img = image::RgbaImage::new(width, height);
                for (i, pixel) in pixmap.pixels().iter().enumerate() {
                    let x = (i % width as usize) as u32;
                    let y = (i / width as usize) as u32;
                    let color = tiny_skia::Color::from_rgba8(
                        pixel.red(),
                        pixel.green(),
                        pixel.blue(),
                        pixel.alpha(),
                    );
                    rgba_img.put_pixel(
                        x,
                        y,
                        image::Rgba([
                            (color.red() * 255.0) as u8,
                            (color.green() * 255.0) as u8,
                            (color.blue() * 255.0) as u8,
                            (color.alpha() * 255.0) as u8,
                        ]),
                    );
                }
                // Encode and download
                let data = match self.export_format.as_str() {
                    "PNG" => {
                        let mut buffer = Vec::new();
                        rgba_img
                            .write_to(
                                &mut std::io::Cursor::new(&mut buffer),
                                image::ImageFormat::Png,
                            )
                            .unwrap();
                        buffer
                    }
                    "JPEG" => {
                        let mut buffer = Vec::new();
                        rgba_img
                            .write_to(
                                &mut std::io::Cursor::new(&mut buffer),
                                image::ImageFormat::Jpeg,
                            )
                            .unwrap();
                        buffer
                    }
                    _ => return,
                };
                web_sys::console::log_1(&format!("Data length: {}", data.len()).into());
                self.download_image(&data, &self.export_format.to_lowercase());
            } else {
                web_sys::console::log_1(&"Failed to load image".into());
            }
        } else {
            web_sys::console::log_1(&"No image bytes".into());
        }
    }

    fn download_image(&self, data: &[u8], format: &str) {
        web_sys::console::log_1(&"Creating blob".into());
        let bag = web_sys::BlobPropertyBag::new();
        bag.set_type(&format!("image/{}", format));
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &js_sys::Array::of1(&js_sys::Uint8Array::from(data)),
            &bag,
        )
        .unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        web_sys::console::log_1(&format!("Blob URL: {}", url).into());
        let document = web_sys::window().unwrap().document().unwrap();
        let a = document
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        a.set_attribute("href", &url).unwrap();
        a.set_attribute("download", &format!("exported.{}", format))
            .unwrap();
        a.click();
        web_sys::Url::revoke_object_url(&url).unwrap();
        web_sys::console::log_1(&"Download initiated".into());
    }
}
