use crate::state::{ExportResizeMode, UiState};
use egui;

pub fn show_export_dialog(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    image_bytes: Option<&Vec<u8>>,
    _on_export: impl FnMut(),
) -> bool {
    let mut should_export = false;
    if ui_state.show_export_dialog {
        let mut open = true;
        egui::Window::new("エクスポート")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("出力フォーマット:");
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(ui_state.export_format == "PNG", "PNG")
                        .clicked()
                    {
                        ui_state.export_format = "PNG".to_string();
                    }
                    if ui
                        .selectable_label(ui_state.export_format == "JPEG", "JPEG")
                        .clicked()
                    {
                        ui_state.export_format = "JPEG".to_string();
                    }
                });

                ui.separator();

                ui.label("リサイズ設定:");
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            ui_state.export_resize_mode == ExportResizeMode::Percentage,
                            "パーセンテージ (%)",
                        )
                        .clicked()
                    {
                        ui_state.export_resize_mode = ExportResizeMode::Percentage;
                    }
                    if ui
                        .selectable_label(
                            ui_state.export_resize_mode == ExportResizeMode::Pixels,
                            "ピクセル数",
                        )
                        .clicked()
                    {
                        ui_state.export_resize_mode = ExportResizeMode::Pixels;
                    }
                });

                match ui_state.export_resize_mode {
                    ExportResizeMode::Percentage => {
                        ui.horizontal(|ui| {
                            ui.label("サイズ (%):");
                            ui.add(
                                egui::Slider::new(&mut ui_state.export_resize_percentage, 1..=500)
                                    .text("%"),
                            );
                        });
                    }
                    ExportResizeMode::Pixels => {
                        ui.horizontal(|ui| {
                            ui.label("メガピクセル (Mpx):");
                            ui.add(
                                egui::DragValue::new(&mut ui_state.export_resize_pixels)
                                    .range(1..=30),
                            );
                        });
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    let can_export = image_bytes.is_some();
                    if ui
                        .add_enabled(can_export, egui::Button::new("エクスポート"))
                        .clicked()
                    {
                        should_export = true;
                        ui_state.show_export_dialog = false;
                    }
                    if ui.button("キャンセル").clicked() {
                        ui_state.show_export_dialog = false;
                    }
                });
                if image_bytes.is_none() {
                    ui.label("画像をロードしてください。");
                }
            });
        if !open {
            ui_state.show_export_dialog = false;
        }
    }
    should_export
}
