use crate::error::ProcessError;
use crate::graphics::{create_watermark_image, overlay_watermark};

use image::Rgba;
use indicatif::ProgressBar;
use log::debug;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use walkdir::WalkDir;

mod error;
mod graphics;

/// Apply recursively a watermark.
///
/// Input `folder` will be traversed, output data will be written in `target_dir`.
/// The watermark is customized through the `Config` struct.
/// The choice of which files/dirs are read or skipped is defined in `Rules` struct.
/// The progression is reported through a given `ProgressBar` struct.
///
/// The processing is multithreaded thanks to `rayon` crate
pub fn spread_watermark<P: AsRef<Path> + std::fmt::Debug + std::marker::Sync>(
    folder: &P,
    target_dir: &P,
    cfg: &Config,
    rules: &Rules,
    progress: Option<&ProgressBar>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !folder.as_ref().is_dir() {
        return Err(Box::new(ProcessError::FsError(format!(
            "Path {:?} is not a directory as required",
            folder
        ))));
    }

    let watermark_img = create_watermark_image(cfg)?;

    let counter = AtomicU64::new(0);
    let entries = WalkDir::new(folder).into_iter().collect::<Vec<_>>();
    let nb_entries = entries.len() as u64;
    if let Some(progress) = progress {
        progress.set_length(nb_entries);
    }

    // handle directories first
    entries
        .iter()
        .filter(|entry| {
            let entry = entry.as_ref().expect("can't get a ref on the entry");
            entry.path().is_dir()
        })
        .for_each(|entry| {
            let path = entry.as_ref().expect("can't get a ref on the entry").path();
            let relative_path = path.strip_prefix(folder).expect("can't strip prefix");
            let new_dir = target_dir.as_ref().join(relative_path);
            fs::create_dir_all(new_dir).expect("error creating dir");
            counter.fetch_add(1, Ordering::Relaxed);
        });

    if let Some(progress) = progress {
        let c = counter.fetch_add(1, Ordering::Relaxed);
        progress.set_position(c);
    }

    // handle files
    entries
        .into_par_iter()
        .filter(|entry| {
            let entry = entry.as_ref().expect("can't get a ref on the entry");
            !entry.path().is_dir()
        })
        .for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative_path = path.strip_prefix(folder).expect("can't strip prefix");
            let target_path = target_dir.as_ref().join(relative_path);

            debug!("entry: {}", path.to_string_lossy());

            if rules.is_file_qualified(&path) {
                debug!("watermarking {:?}", path);

                overlay_watermark(path, target_path.as_path(), &watermark_img)
                    .expect("error watermarking");
            } else {
                debug!("copying {:?}", path);

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

/// Rules to watermark files.
/// Using this struct you can select which
/// files will be watermarked or not, and
/// which folders will be traversed.
#[derive(Debug)]
pub struct Rules {
    /// Name of directories to exclude
    /// if path contains a name from this list,
    /// content of dir will not be watermarked
    /// i.e.: "/some/path/.hidden/pic.jpg" won't be processed
    /// if ".hidden" is part of `excluded_dirs`
    pub excluded_dirs: Vec<String>,
    /// Name of files to exclude
    /// if filename starts with a name from this list,
    /// image file will not be watermarked
    /// i.e.: "/some/path/background.png" won't be watermarked
    /// if "back" is part of `excluded_files`
    pub excluded_files: Vec<String>,
    /// Extensions allowed to be watermarked
    /// i.e.: ["png", "jpg", ...]
    pub authorized_extensions: Vec<String>,
}

impl Rules {
    /// File is qualified if it is not part of excluded file list
    /// and if its extension is authorized.
    pub fn is_file_qualified<P: AsRef<Path>>(&self, path: &P) -> bool {
        let path = path.as_ref();
        if self.excluded_dirs.iter().any(|dir| {
            path.to_str()
                .expect("unable to convert path to str")
                .contains(dir)
        }) {
            return false;
        }

        if let Some(extension) = path.extension() {
            !self.excluded_files.iter().any(|filename| {
                path.file_name()
                    .expect("can't retrieve filename")
                    .to_str()
                    .expect("unable to convert filename to str")
                    .starts_with(filename.as_str())
            }) && self
                .authorized_extensions
                .iter()
                .any(|ext| ext.as_str() == extension)
        } else {
            false
        }
    }
}

/// Customization of the watermark.
/// Basically you can choose the `text`,
/// the `color` and the `scale` (size) of
/// the watermark that will be applied
#[derive(Debug)]
pub struct Config {
    pub text: String,
    pub color: image::Rgba<u8>,
    pub scale: rusttype::Scale,
}

impl Default for Config {
    fn default() -> Self {
        // scale
        let height = 28.0;
        let scale = rusttype::Scale {
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

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
