use image::imageops::{overlay, FilterType};
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use rusttype::Font;
use std::path::Path;

use crate::Config;

pub fn create_watermark_image(cfg: &Config) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let mut img: RgbaImage = ImageBuffer::new(500, 500);

    // font for watermark
    let font_bytes = include_bytes!("../fonts/Roboto-Bold.ttf");
    let font = Font::try_from_bytes(font_bytes).expect("font is not valid");

    draw_text_mut(&mut img, cfg.color, 0, 210, cfg.scale, &font, &cfg.text);

    // rotate to render text in diagonal
    img = rotate_about_center(&img, 0.8, Interpolation::Bicubic, Rgba([255, 0, 0, 0]));
    Ok(img)
}

pub fn overlay_watermark(
    src: &Path,
    dst: &Path,
    watermark_img: &RgbaImage,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = image::open(
        src.to_str()
            .expect("given src path can't be converted to str"),
    )?;
    img = img.resize_exact(500, 500, FilterType::Nearest);
    overlay(&mut img, watermark_img, 0, 0);
    img.save(
        dst.to_str()
            .expect("given dst path can't be converted to str"),
    )?;
    Ok(())
}
