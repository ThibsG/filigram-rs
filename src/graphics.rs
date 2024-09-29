use ab_glyph::FontRef;
use image::imageops::{overlay, FilterType};
use image::ImageReader;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use std::path::Path;

use crate::config::Config;

pub fn create_watermark_image(cfg: &Config) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let mut img: RgbaImage = ImageBuffer::new(500, 500);

    // font for watermark
    let font_bytes = include_bytes!("../fonts/Roboto-Bold.ttf");
    let font = FontRef::try_from_slice(font_bytes)?;

    draw_text_mut(&mut img, cfg.color, 0, 210, cfg.scale, &font, &cfg.text);

    // rotate to render text in diagonal
    img = rotate_about_center(&img, 0.8, Interpolation::Bicubic, Rgba([255, 0, 0, 0]));
    Ok(img)
}

pub fn overlay_watermark<P: AsRef<Path>>(
    src: P,
    dst: P,
    watermark_img: &RgbaImage,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open(src)?.decode()?;
    img = img.resize_exact(500, 500, FilterType::Nearest);
    overlay(&mut img, watermark_img, 0, 0);
    img.save(dst)?;
    Ok(())
}
