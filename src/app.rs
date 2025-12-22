use crate::canvas_items::*;
use crate::drawing::{ItemRenderer, PreviewRenderer, ShapeFactory};
use crate::export::{DownloadHandler, ImageExporter};
use crate::state::{AppMode, DrawingState, SelectionState, UiState};
use crate::ui;
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
    rectangles: Vec<CanvasItem>,

    // State management
    drawing_state: DrawingState,
    ui_state: UiState,
    selection_state: SelectionState,
}

impl Default for AnnotoApp {
    fn default() -> Self {
        Self {
            image_texture: None,
            image_bytes: None,
            rectangles: Vec::new(),
            drawing_state: DrawingState::default(),
            ui_state: UiState::default(),
            selection_state: SelectionState::default(),
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
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "NotoSansRegular".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("NotoSansRegular".to_owned());
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

        ui::render_top_panel(
            ctx,
            &mut self.drawing_state,
            &mut self.ui_state,
            Self::open_file_dialog,
        );

        self.sync_ui_from_selection();

        ui::render_side_panel(
            ctx,
            &mut self.drawing_state,
            &self.ui_state,
            self.selection_state.selected_item,
            &self.rectangles,
            || {},
        );

        // Render central panel with closures
        self.render_central_panel_with_closures(ctx);

        let should_export =
            ui::show_export_dialog(ctx, &mut self.ui_state, self.image_bytes.as_ref(), || {});

        if should_export {
            self.export_image();
        }

        // Update selected item after UI rendering
        self.update_selected_item();

        self.handle_keyboard_events(ctx);
    }
}

impl AnnotoApp {
    fn render_central_panel_with_closures(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.image_texture.clone() {
                let scale = self.drawing_state.zoom / 100.0;

                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let pointer_pos = ui.input(|i| i.pointer.hover_pos());

                        let scaled_size = texture.size_vec2() * scale;
                        let image_response =
                            ui.allocate_response(scaled_size, egui::Sense::click_and_drag());
                        let mut image_rect = image_response.rect;

                        if self.ui_state.mode == AppMode::View {
                            image_rect = image_rect.translate(self.ui_state.pan_offset);
                        }

                        ui.painter().image(
                            texture.id(),
                            image_rect,
                            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(1.0)),
                            egui::Color32::WHITE,
                        );

                        if let Some(pos) = pointer_pos {
                            if image_rect.contains(pos) {
                                let cursor_in_image = (pos - image_rect.min) / scale;
                                self.ui_state.cursor_pos =
                                    Some(egui::Pos2::new(cursor_in_image.x, cursor_in_image.y));
                            }
                        } else {
                            self.ui_state.cursor_pos = None;
                        }

                        if self.ui_state.mode == AppMode::Drawing {
                            self.handle_drawing_mode(ui, &image_response, image_rect, scale);
                            ItemRenderer::render_existing_items(
                                ui,
                                &mut self.rectangles,
                                image_rect,
                                scale,
                            );
                            PreviewRenderer::render_drag_preview(
                                ui,
                                &self.drawing_state,
                                image_rect,
                                scale,
                            );
                            ItemRenderer::render_handles(
                                ui,
                                self.selection_state.selected_item,
                                &mut self.selection_state.selected_handle,
                                &mut self.rectangles,
                                image_rect,
                                scale,
                            );

                            let mut hovering_index = None;
                            if let Some(pos) = pointer_pos {
                                if image_rect.contains(pos) {
                                    for (i, item) in self.rectangles.iter().enumerate() {
                                        if item.hit_test(pos, image_rect, scale) {
                                            hovering_index = Some(i);
                                            break;
                                        }
                                    }
                                    if hovering_index.is_some() {
                                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                                    } else {
                                        ui.output_mut(|o| {
                                            o.cursor_icon = egui::CursorIcon::Default
                                        });
                                    }
                                }
                            }

                            if image_response.clicked() {
                                if let Some(idx) = hovering_index {
                                    self.selection_state.selected_item = Some(idx);
                                    self.selection_state.selected_handle = None;
                                } else {
                                    self.selection_state.selected_item = None;
                                    self.selection_state.selected_handle = None;
                                }
                            }

                            if image_response.drag_started() {
                                if let Some(idx) = hovering_index {
                                    self.selection_state.selected_item = Some(idx);
                                    self.selection_state.selected_handle = None;
                                } else {
                                    self.selection_state.selected_item = None;
                                    self.selection_state.selected_handle = None;
                                }
                            }

                            if let Some(selected_idx) = self.selection_state.selected_item {
                                if image_response.dragged()
                                    && self.selection_state.selected_handle.is_none()
                                {
                                    let drag_delta = image_response.drag_delta() / scale;
                                    if let Some(item) = self.rectangles.get_mut(selected_idx) {
                                        item.translate(drag_delta);
                                    }
                                }
                            }
                        } else {
                            ItemRenderer::render_existing_items(
                                ui,
                                &mut self.rectangles,
                                image_rect,
                                scale,
                            );

                            if image_response.dragged() {
                                self.ui_state.pan_offset += image_response.drag_delta();
                            }

                            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                            if scroll_delta != 0.0 {
                                let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                                self.drawing_state.zoom =
                                    (self.drawing_state.zoom * zoom_factor).clamp(1.0, 500.0);
                            }
                        }
                    });
            }
        });
    }

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
                let hovering_index = self
                    .rectangles
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.hit_test(pos, image_rect, scale))
                    .map(|(i, _)| i);
                if hovering_index.is_some() {
                    return;
                }

                if image_response.drag_started() {
                    self.drawing_state.drag_start = Some(pos);
                }
                if image_response.drag_stopped() {
                    if let Some(start) = self.drawing_state.drag_start {
                        let end = pos;
                        if let Some(shape) = ShapeFactory::create_shape_from_drag(
                            self.drawing_state.current_tool,
                            start,
                            end,
                            image_rect,
                            scale,
                            &self.drawing_state,
                        ) {
                            self.rectangles.push(shape);
                        }
                        self.drawing_state.drag_start = None;
                    }
                }
            }
        }
    }

    fn export_image(&self) {
        web_sys::console::log_1(&"Exporting image".into());
        if let Some(image_bytes) = &self.image_bytes {
            match ImageExporter::export_image(
                image_bytes,
                &self.rectangles,
                &self.ui_state.export_format,
            ) {
                Ok(data) => {
                    DownloadHandler::download_image(
                        &data,
                        &self.ui_state.export_format.to_lowercase(),
                    );
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Export error: {}", e).into());
                }
            }
        } else {
            web_sys::console::log_1(&"No image bytes".into());
        }
    }

    fn sync_ui_from_selection(&mut self) {
        if let Some(idx) = self.selection_state.selected_item {
            if let Some(item) = self.rectangles.get(idx) {
                if let Some(w) = item.get_stroke_width() {
                    self.drawing_state.stroke_width = w;
                }
                if let Some(c) = item.get_stroke_color() {
                    self.drawing_state.stroke_color = c;
                }
                if let Some(c) = item.get_fill_color() {
                    self.drawing_state.fill_color = c;
                }
                if let Some(r) = item.get_rounding() {
                    self.drawing_state.rounding = r;
                }
                if let CanvasItem::Mosaic(mosaic) = item {
                    self.drawing_state.mosaic_granularity = mosaic.get_granularity();
                }
            }
        }
    }

    fn update_selected_item(&mut self) {
        if let Some(idx) = self.selection_state.selected_item {
            if let Some(item) = self.rectangles.get_mut(idx) {
                item.set_stroke_width(self.drawing_state.stroke_width);
                item.set_stroke_color(self.drawing_state.stroke_color);
                item.set_fill_color(self.drawing_state.fill_color);
                item.set_rounding(self.drawing_state.rounding);
            }
        }
    }

    fn handle_keyboard_events(&mut self, ctx: &egui::Context) {
        if self.ui_state.mode != AppMode::Drawing {
            return;
        }
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                if let Some(idx) = self.selection_state.selected_item {
                    self.rectangles.remove(idx);
                    self.selection_state.selected_item = None;
                    self.selection_state.selected_handle = None;
                }
            }
        });
    }
}
