use crate::canvas_items::CanvasItem;
use image::ImageFormat;
use std::io::Cursor;
use web_sys;

pub struct ImageExporter;

impl ImageExporter {
    /// 画像にアイテムを描画してエクスポート
    pub fn export_image(
        image_bytes: &[u8],
        rectangles: &[CanvasItem],
        format: &str,
    ) -> Result<Vec<u8>, String> {
        if let Ok(img) = image::load_from_memory(image_bytes) {
            let rgba_img = img.to_rgba8();
            let width = rgba_img.width() as u32;
            let height = rgba_img.height() as u32;
            let mut pixmap =
                tiny_skia::Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

            // Copy image data
            for (i, pixel) in rgba_img.pixels().enumerate() {
                let color = tiny_skia::Color::from_rgba8(pixel[0], pixel[1], pixel[2], pixel[3]);
                pixmap.pixels_mut()[i] = color.premultiply().to_color_u8();
            }

            // Draw shapes on the pixmap
            for item in rectangles {
                match item {
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
