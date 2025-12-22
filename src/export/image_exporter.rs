use crate::canvas_items::CanvasItem;
use crate::state::ExportResizeMode;
use image::ImageFormat;
use std::io::Cursor;
use web_sys;

pub struct ImageExporter;

impl ImageExporter {
    /// リサイズ設定付きでエクスポート
    pub fn export_image_with_resize(
        image_bytes: &[u8],
        rectangles: &[CanvasItem],
        format: &str,
        resize_mode: ExportResizeMode,
        resize_percentage: u32,
        resize_pixels: u32,
    ) -> Result<Vec<u8>, String> {
        if let Ok(img) = image::load_from_memory(image_bytes) {
            let rgba_img = img.to_rgba8();
            let original_width = rgba_img.width() as u32;
            let original_height = rgba_img.height() as u32;
            let mut width = original_width;
            let mut height = original_height;

            // リサイズ計算
            let scale_factor = match resize_mode {
                ExportResizeMode::Percentage => {
                    let scale = resize_percentage as f32 / 100.0;
                    width = (width as f32 * scale).max(1.0) as u32;
                    height = (height as f32 * scale).max(1.0) as u32;
                    scale
                }
                ExportResizeMode::Pixels => {
                    // Mpx（メガピクセル）単位で指定された値から画像サイズを計算
                    // 1 Mpx = 1,000,000 pixels
                    let target_pixels = (resize_pixels as f64) * 1_000_000.0;
                    let original_width_f = original_width as f64;
                    let original_height_f = original_height as f64;
                    let aspect_ratio = original_height_f / original_width_f;

                    // width * height = target_pixels
                    // height = width * aspect_ratio
                    // width * (width * aspect_ratio) = target_pixels
                    // width^2 = target_pixels / aspect_ratio
                    let new_width_f = (target_pixels / aspect_ratio).sqrt();
                    width = new_width_f.max(1.0) as u32;
                    height = (new_width_f * aspect_ratio).max(1.0) as u32;
                    width as f32 / original_width as f32
                }
            };

            // リサイズ処理
            let resized_img = if width != original_width || height != original_height {
                image::imageops::resize(
                    &rgba_img,
                    width,
                    height,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                rgba_img.clone()
            };

            let mut pixmap =
                tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

            // Copy image data
            for (i, pixel) in resized_img.pixels().enumerate() {
                let color = tiny_skia::Color::from_rgba8(pixel[0], pixel[1], pixel[2], pixel[3]);
                pixmap.pixels_mut()[i] = color.premultiply().to_color_u8();
            }

            // Draw shapes on the pixmap (with scaling)
            for item in rectangles {
                let scaled_item = item.scale(scale_factor);
                match scaled_item {
                    CanvasItem::StrokeRect(rect) => rect.draw_on_pixmap(&mut pixmap),
                    CanvasItem::FilledRect(rect) => rect.draw_on_pixmap(&mut pixmap),
                    CanvasItem::Arrow(arrow) => arrow.draw_on_pixmap(&mut pixmap),
                    CanvasItem::Line(line) => line.draw_on_pixmap(&mut pixmap),
                    CanvasItem::Mosaic(mosaic) => {
                        mosaic.draw_on_pixmap(&mut pixmap, image_bytes);
                    }
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

            // Encode
            let data = match format {
                "PNG" => {
                    let mut buffer = Vec::new();
                    rgba_img
                        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
                        .map_err(|e| format!("PNG encoding failed: {}", e))?;
                    buffer
                }
                "JPEG" => {
                    let mut buffer = Vec::new();
                    rgba_img
                        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg)
                        .map_err(|e| format!("JPEG encoding failed: {}", e))?;
                    buffer
                }
                _ => return Err("Unsupported format".to_string()),
            };

            web_sys::console::log_1(&format!("Data length: {}", data.len()).into());
            Ok(data)
        } else {
            Err("Failed to load image".to_string())
        }
    }
}
