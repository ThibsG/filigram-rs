use core::sync::atomic::{AtomicU64, Ordering};
use indicatif::ProgressBar;
use log::{debug, error};
use rayon::prelude::*;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub mod config;
mod graphics;
pub mod rules;

use crate::graphics::{create_watermark_image, overlay_watermark};

pub use crate::config::Config;
pub use crate::rules::Rules;

/// Apply recursively a watermark.
///
/// Input `folder` will be traversed, output data will be written in `target_dir`.
/// The watermark is customized through the `Config` struct.
/// The choice of which files/dirs are read or skipped is defined in `Rules` struct.
/// The progression is reported through a given `ProgressBar` struct.
///
/// The processing is multithreaded thanks to `rayon` crate
pub fn spread_watermark<P: AsRef<Path> + std::fmt::Debug + Sync>(
    folder: &P,
    target_dir: &P,
    cfg: &Config,
    rules: &Rules,
    progress: Option<&ProgressBar>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !folder.as_ref().is_dir() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path {folder:?} is not a directory as required"),
        )));
    }

    let watermark_img = create_watermark_image(cfg)?;

    let counter = AtomicU64::new(0);
    let entries = WalkDir::new(folder)
        .into_iter()
        .collect::<Result<Vec<walkdir::DirEntry>, walkdir::Error>>()?;
    let nb_entries = entries.len() as u64;
    if let Some(progress) = progress {
        progress.set_length(nb_entries);
    }

    // create directory structure first
    entries
        .par_iter()
        .filter(|entry| entry.path().is_dir())
        .for_each(|entry| {
            let path = entry.path();
            let relative_path = path.strip_prefix(folder).expect("can't strip prefix");
            let new_dir = target_dir.as_ref().join(relative_path);
            fs::create_dir_all(new_dir).expect("error creating dir");

            if progress.is_some() {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });

    if let Some(progress) = progress {
        let c = counter.fetch_add(1, Ordering::Relaxed);
        progress.set_position(c);
    }

    // handle files
    entries
        .into_par_iter()
        .filter(|entry| !entry.path().is_dir())
        .for_each(|entry| {
            let path = entry.path();
            debug!("entry: {path:?}");

            let relative_path = path.strip_prefix(folder).expect("can't strip prefix");
            let target_path = target_dir.as_ref().join(relative_path);

            if rules.is_file_qualified(&path) {
                debug!("watermarking {path:?}");

                if let Err(e) = overlay_watermark(&path, &target_path, &watermark_img) {
                    error!("Error watermarking: {:?} - {}", path, e.to_string());
                }
            } else {
                debug!("copying {path:?}");

                fs::copy(path, target_path).expect("error copying a file");
            }

            // Progress update
            if let Some(progress) = progress {
                let c = counter.fetch_add(1, Ordering::Relaxed);
                if nb_entries < 1000 || c % 100 == 0 {
                    progress.set_position(c);
                }
            }
        });

    Ok(())
}
