use filigram_rs::{config::Config, rules::Rules, spread_watermark};
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("filigram-rs program");

    if std::fs::read_dir("./result").is_ok() {
        std::fs::remove_dir_all("./result")?;
    }
    std::fs::create_dir("./result")?;

    let input = PathBuf::from("./data/input").canonicalize()?;
    let target_dir = PathBuf::from("./result").canonicalize()?;

    info!("From: {:?}", input);
    info!("to: {:?}", target_dir);

    let progress = ProgressBar::new(0).with_style(progress_style());
    progress.enable_steady_tick(250);

    let rules = Rules {
        excluded_dirs: vec![".hidden".to_string()],
        authorized_extensions: vec![
            "jpg".to_string(),
            "jpeg".to_string(),
            "png".to_string(),
            "bmp".to_string(),
            "gif".to_string(),
        ],
        excluded_files: vec!["background".to_string()],
    };

    // default parameters
    let cfg = Config::new();

    spread_watermark(&input, &target_dir, &cfg, &rules, Some(&progress))?;

    progress.finish();

    Ok(())
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.blue}] ({eta_precise} left)")
        .progress_chars("#>-")
}
