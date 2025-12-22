use crate::canvas_items::*;
use crate::drawing::{ItemRenderer, PreviewRenderer, ShapeFactory};
use crate::export::{DownloadHandler, ImageExporter};
use crate::state::{DrawingState, SelectionState, UiState};
use crate::touch_handler::get_current_touches;
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

                        // パンオフセットを適用（View モードと Drawing モード両方で）
                        image_rect = image_rect.translate(self.ui_state.pan_offset);

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

                        // タッチイベント処理
                        self.handle_touch_events(image_rect, scale);

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
                        let should_delete = ItemRenderer::render_handles(
                            ui,
                            self.selection_state.selected_item,
                            &mut self.selection_state.selected_handle,
                            &mut self.rectangles,
                            image_rect,
                            scale,
                        );

                        if should_delete {
                            if let Some(idx) = self.selection_state.selected_item {
                                self.rectangles.remove(idx);
                                self.selection_state.selected_item = None;
                                self.selection_state.selected_handle = None;
                            }
                        }

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
                                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Default);
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

        // マルチタッチ中は描画処理をスキップ
        if self.ui_state.touch_points.len() >= 2 {
            return;
        }

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

    fn handle_touch_events(&mut self, image_rect: egui::Rect, scale: f32) {
        // タッチポイントを更新（touch_handler から取得）
        self.ui_state.prev_touch_points = self.ui_state.touch_points.clone();
        self.ui_state.touch_points = get_current_touches();

        // 2本指以上のタッチがある場合
        if self.ui_state.touch_points.len() >= 2 {
            // 前フレームでも2本指以上あった場合
            if self.ui_state.prev_touch_points.len() >= 2 {
                // 2本指の移動距離を計算してパンニングとピンチズームを判定
                let curr_p1 = self.ui_state.touch_points[0].pos;
                let curr_p2 = self.ui_state.touch_points[1].pos;
                let prev_p1 = self.ui_state.prev_touch_points[0].pos;
                let prev_p2 = self.ui_state.prev_touch_points[1].pos;

                let curr_distance = (curr_p2 - curr_p1).length();
                let prev_distance = (prev_p2 - prev_p1).length();

                // 距離の変化が大きい場合はピンチズーム
                let distance_change = (curr_distance - prev_distance).abs();
                if distance_change > 3.0 {
                    self.handle_pinch_zoom(image_rect, scale);
                } else {
                    // 距離の変化が小さい場合はパンニング
                    self.handle_two_finger_pan();
                }
            }
        }
    }

    fn handle_two_finger_pan(&mut self) {
        let curr_points = &self.ui_state.touch_points;
        let prev_points = &self.ui_state.prev_touch_points;

        if curr_points.len() < 2 || prev_points.len() < 2 {
            return;
        }

        // 2本指の中点を計算
        let curr_center = egui::Pos2::new(
            (curr_points[0].pos.x + curr_points[1].pos.x) / 2.0,
            (curr_points[0].pos.y + curr_points[1].pos.y) / 2.0,
        );

        let prev_center = egui::Pos2::new(
            (prev_points[0].pos.x + prev_points[1].pos.x) / 2.0,
            (prev_points[0].pos.y + prev_points[1].pos.y) / 2.0,
        );

        // 中点の移動量を計算
        let pan_delta = curr_center - prev_center;

        // パンオフセットを更新
        self.ui_state.pan_offset += pan_delta;
    }

    fn handle_pinch_zoom(&mut self, image_rect: egui::Rect, scale: f32) {
        let curr_points = &self.ui_state.touch_points;
        let prev_points = &self.ui_state.prev_touch_points;

        if curr_points.len() < 2 || prev_points.len() < 2 {
            return;
        }

        // 現在の2本指の距離を計算
        let curr_p1 = curr_points[0].pos;
        let curr_p2 = curr_points[1].pos;
        let curr_distance = (curr_p2 - curr_p1).length();

        // 前フレームの2本指の距離を計算
        let prev_p1 = prev_points[0].pos;
        let prev_p2 = prev_points[1].pos;
        let prev_distance = (prev_p2 - prev_p1).length();

        if prev_distance < 1.0 {
            return;
        }

        // ピンチの中点を計算（現在）
        let curr_center =
            egui::Pos2::new((curr_p1.x + curr_p2.x) / 2.0, (curr_p1.y + curr_p2.y) / 2.0);

        // ズーム比率を計算
        let zoom_ratio = curr_distance / prev_distance;

        // 新しいズーム値を計算
        let new_zoom = (self.drawing_state.zoom * zoom_ratio).clamp(1.0, 500.0);

        // ピンチ中心がスクリーン上のどこにあるかを計算
        // 画像座標系でのピンチ中心
        if image_rect.contains(curr_center) {
            // ピンチ中心のスクリーン座標から画像座標への変換
            let pinch_screen_offset = curr_center - image_rect.min;
            let pinch_image_pos = pinch_screen_offset / scale;

            // ズーム前後での画像座標の変化を計算
            let zoom_change = new_zoom / self.drawing_state.zoom;

            // パンオフセットを調整して、ピンチ中心が固定されるようにする
            // ピンチ中心が画面上の同じ位置に留まるようにオフセットを調整
            let offset_adjustment = pinch_image_pos * (1.0 - zoom_change) * scale;
            self.ui_state.pan_offset += offset_adjustment;

            self.drawing_state.zoom = new_zoom;
        }
    }
}
