use ab_glyph::PxScale;
use image::Rgba;

/// Customization of the watermark.
/// Basically you can choose the `text`,
/// the `color` and the `scale` (size) of
/// the watermark that will be applied
#[derive(Debug)]
pub struct Config {
    pub text: String,
    pub color: image::Rgba<u8>,
    pub scale: PxScale,
}

impl Default for Config {
    fn default() -> Self {
        // scale
        let height = 28.0;
        let scale = PxScale {
            x: height * 2.3,
            y: height * 2.3,
        };

        Self {
            text: "© Copyright Filigram".to_owned(),
            color: Rgba([0_u8, 0_u8, 0_u8, 110_u8]),
            scale,
        }
    }
}
