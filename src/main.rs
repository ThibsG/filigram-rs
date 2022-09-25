use filigram_rs::{config::Config, rules::Rules, spread_watermark};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::{path::PathBuf, time::Duration};

static RESULT_PATH: &str = "./result";
static INPUT_PATH: &str = "./data/input";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "warn,info,error,debug");
    env_logger::init();

    info!("Starting program");

    let input = PathBuf::from(INPUT_PATH).canonicalize()?;
    let target_dir = PathBuf::from(RESULT_PATH).canonicalize()?;

    if target_dir.exists() {
        warn!("removing existing results");
        std::fs::remove_dir_all(&target_dir)?;
    }
    std::fs::create_dir(&target_dir)?;

    info!("from: {input:?}");
    info!("to:   {target_dir:?}");

    let progress = ProgressBar::new(0).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.blue}] ({eta_precise} left)")?
            .progress_chars("#>-"),
    );
    progress.enable_steady_tick(Duration::from_millis(250));

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
