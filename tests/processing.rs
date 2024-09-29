use filigram_rs::{create_watermark_image, overlay_watermark, Config};

macro_rules! run_test {
    ($extension:literal) => {
        let cfg = Config::default();
        std::fs::create_dir("tmp").ok();
        let watermark_img = create_watermark_image(&cfg).unwrap();
        overlay_watermark(
            format!("tests/img/test.{}", $extension),
            format!("tmp/test.{}", $extension),
            &watermark_img,
        )
        .unwrap();
    };
}

#[test]
fn test_jpeg() {
    run_test!("jpg");
}

#[test]
fn test_gif() {
    run_test!("gif");
}

#[test]
fn test_webp() {
    run_test!("webp");
}

#[test]
fn test_bmp() {
    run_test!("bmp");
}
