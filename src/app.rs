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
    drawing_mode: bool,
    rectangles: Vec<CanvasItem>,
    drag_start: Option<egui::Pos2>,
    selected_index: Option<usize>,
    stroke_width: f32,
    stroke_color: egui::Color32,
    fill_enabled: bool,
    fill_color: egui::Color32,
    current_tool: DrawingTool,
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
            current_tool: DrawingTool::StrokeRect,
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
    }

    fn render_side_panel(&mut self, ctx: &egui::Context) {
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
            ui.add_space(16.0);
            if let Some(index) = self.selected_index {
                ui.label("選択アイテム編集");
                if ui.button("削除").clicked() {
                    self.rectangles.remove(index);
                    self.selected_index = None;
                }
                match &mut self.rectangles[index] {
                    CanvasItem::StrokeRect(rect) => {
                        ui.label("線の色:");
                        ui.color_edit_button_srgba(&mut rect.stroke_color);
                        ui.label("線の太さ:");
                        ui.add(
                            egui::DragValue::new(&mut rect.stroke_width)
                                .range(1..=50)
                                .suffix("px"),
                        );
                    }
                    CanvasItem::FilledRect(rect) => {
                        ui.label("塗りつぶし色:");
                        ui.color_edit_button_srgba(&mut rect.filled_color);
                    }
                    CanvasItem::Arrow(arrow) => {
                        ui.label("線の色:");
                        ui.color_edit_button_srgba(&mut arrow.color);
                    }
                }
            }
        });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
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

                self.handle_drawing_mode(ui, &image_response, image_rect, scale);
                self.render_existing_items(ui, image_rect, scale);
                self.render_drag_preview(ui, image_rect, scale);
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
                            match self.current_tool {
                                DrawingTool::StrokeRect => {
                                    let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                                    let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                                    let offset_min = (min - image_rect.min) / scale;
                                    let offset_max = (max - image_rect.min) / scale;
                                    self.rectangles.push(CanvasItem::StrokeRect(StrokeRect {
                                        x1: offset_min.x,
                                        y1: offset_min.y,
                                        x2: offset_max.x,
                                        y2: offset_max.y,
                                        stroke_width: self.stroke_width,
                                        stroke_color: self.stroke_color,
                                    }));
                                }
                                DrawingTool::FilledRect => {
                                    let min = egui::pos2(start.x.min(end.x), start.y.min(end.y));
                                    let max = egui::pos2(start.x.max(end.x), start.y.max(end.y));
                                    let offset_min = (min - image_rect.min) / scale;
                                    let offset_max = (max - image_rect.min) / scale;
                                    self.rectangles.push(CanvasItem::FilledRect(FilledRect {
                                        x1: offset_min.x,
                                        y1: offset_min.y,
                                        x2: offset_max.x,
                                        y2: offset_max.y,
                                        filled_color: self.fill_color,
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
                            }
                            self.drag_start = None;
                        }
                    }
                }
            }
        }
    }

    fn render_existing_items(&mut self, ui: &mut egui::Ui, image_rect: egui::Rect, scale: f32) {
        let mut new_selected = None;
        for (index, item) in self.rectangles.iter_mut().enumerate() {
            let sel = match item {
                CanvasItem::StrokeRect(rect) => rect.render(
                    ui,
                    index,
                    image_rect,
                    scale,
                    self.selected_index,
                    self.drawing_mode,
                ),
                CanvasItem::FilledRect(rect) => rect.render(
                    ui,
                    index,
                    image_rect,
                    scale,
                    self.selected_index,
                    self.drawing_mode,
                ),
                CanvasItem::Arrow(arrow) => arrow.render(
                    ui,
                    index,
                    image_rect,
                    scale,
                    self.selected_index,
                    self.drawing_mode,
                ),
            };
            if let Some(s) = sel {
                new_selected = Some(s);
            }
        }
        if new_selected.is_some() {
            self.selected_index = new_selected;
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

                let mut preview = StrokeRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    stroke_width: self.stroke_width,
                    stroke_color: self.stroke_color,
                };
                let _ = preview.render(ui, 0, image_rect, scale, None, true);
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

                let mut preview = FilledRect {
                    x1: offset_min.x,
                    y1: offset_min.y,
                    x2: offset_max.x,
                    y2: offset_max.y,
                    filled_color: self.fill_color,
                };
                let _ = preview.render(ui, 0, image_rect, scale, None, true);
            }
            DrawingTool::Arrow => {
                let offset_start = (start_world - image_rect.min) / scale;
                let offset_end = (end_world - image_rect.min) / scale;
                let mut preview = Arrow {
                    start_x: offset_start.x,
                    start_y: offset_start.y,
                    end_x: offset_end.x,
                    end_y: offset_end.y,
                    color: self.stroke_color,
                };
                let _ = preview.render(ui, 0, image_rect, scale, None, true);
            }
        }
    }
}
