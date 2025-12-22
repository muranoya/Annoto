use crate::canvas_items::CanvasItem;
use crate::drawing_tool::DrawingTool;
use crate::state::{AppMode, DrawingState, UiState};
use egui;

pub fn render_side_panel(
    ctx: &egui::Context,
    drawing_state: &mut DrawingState,
    ui_state: &UiState,
    selected_item: Option<usize>,
    rectangles: &[CanvasItem],
    mut on_update_selected: impl FnMut(),
) {
    if ui_state.mode != AppMode::Drawing {
        return;
    }
    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.label("描画ツール");
        if ui
            .selectable_label(
                matches!(drawing_state.current_tool, DrawingTool::StrokeRect),
                "四角形",
            )
            .clicked()
        {
            drawing_state.current_tool = DrawingTool::StrokeRect;
        }
        if ui
            .selectable_label(
                matches!(drawing_state.current_tool, DrawingTool::FilledRect),
                "塗りつぶし四角形",
            )
            .clicked()
        {
            drawing_state.current_tool = DrawingTool::FilledRect;
        }
        if ui
            .selectable_label(
                matches!(drawing_state.current_tool, DrawingTool::Arrow),
                "矢印",
            )
            .clicked()
        {
            drawing_state.current_tool = DrawingTool::Arrow;
        }
        if ui
            .selectable_label(
                matches!(drawing_state.current_tool, DrawingTool::Line),
                "直線",
            )
            .clicked()
        {
            drawing_state.current_tool = DrawingTool::Line;
        }
        if ui
            .selectable_label(
                matches!(drawing_state.current_tool, DrawingTool::Mosaic),
                "モザイク",
            )
            .clicked()
        {
            drawing_state.current_tool = DrawingTool::Mosaic;
        }
        ui.add_space(16.0);

        let tool_type = if let Some(idx) = selected_item {
            if let Some(item) = rectangles.get(idx) {
                match item {
                    CanvasItem::StrokeRect(_) => "StrokeRect",
                    CanvasItem::FilledRect(_) => "FilledRect",
                    CanvasItem::Arrow(_) => "Arrow",
                    CanvasItem::Line(_) => "Line",
                    CanvasItem::Mosaic(_) => "Mosaic",
                }
            } else {
                ""
            }
        } else {
            match drawing_state.current_tool {
                DrawingTool::StrokeRect => "StrokeRect",
                DrawingTool::FilledRect => "FilledRect",
                DrawingTool::Arrow => "Arrow",
                DrawingTool::Line => "Line",
                DrawingTool::Mosaic => "Mosaic",
            }
        };

        if matches!(tool_type, "StrokeRect" | "Line") {
            ui.label("線の太さ:");
            if ui
                .add(
                    egui::DragValue::new(&mut drawing_state.stroke_width)
                        .range(1..=50)
                        .suffix("px"),
                )
                .changed()
            {
                on_update_selected();
            }
            ui.add_space(16.0);
        }

        if matches!(tool_type, "Mosaic") {
            ui.label("モザイク粒度:");
            if ui
                .add(
                    egui::DragValue::new(&mut drawing_state.mosaic_granularity)
                        .range(1..=100)
                        .suffix("px"),
                )
                .changed()
            {
                on_update_selected();
            }
            ui.add_space(16.0);
        }

        if !tool_type.is_empty() {
            ui.label("線の色:");
            if ui
                .color_edit_button_srgba(&mut drawing_state.stroke_color)
                .changed()
            {
                on_update_selected();
            }
        }

        if matches!(tool_type, "FilledRect") {
            ui.add_space(16.0);
            ui.label("塗りつぶし色:");
            if ui
                .color_edit_button_srgba(&mut drawing_state.fill_color)
                .changed()
            {
                on_update_selected();
            }
        }

        if matches!(tool_type, "StrokeRect" | "FilledRect") {
            ui.add_space(16.0);
            ui.label("角の丸め:");
            if ui
                .add(
                    egui::DragValue::new(&mut drawing_state.rounding)
                        .range(0..=255)
                        .suffix("px"),
                )
                .changed()
            {
                on_update_selected();
            }
        }
    });
}
