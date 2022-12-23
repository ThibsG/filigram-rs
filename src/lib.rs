use core::sync::atomic::{AtomicU64, Ordering};
use img_parts::{jpeg::Jpeg, png::Png};
use img_parts::{ImageEXIF, ImageICC};
use log::{debug, error};
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub mod config;
mod graphics;
pub mod rules;

use crate::graphics::{create_watermark_image, overlay_watermark};

pub use crate::config::Config;
pub use crate::rules::Rules;
pub use indicatif;

use indicatif::ProgressBar;

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
                } else {
                    recopy_metadata(&path, &target_path.as_path())
                        .expect("cannot recopy properties");
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

// Recopy file's metadata from original file (`from`) to watermarked one (`to`)
fn recopy_metadata<P: AsRef<Path> + ?Sized + std::fmt::Debug>(
    from: &P,
    to: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = fs::read(from).expect("cannot read image");
    let output = fs::read(to).expect("cannot read target image");

    match from
        .as_ref()
        .extension()
        .unwrap()
        .to_string_lossy()
        .to_lowercase()
        .as_str()
    {
        "png" => {
            let input_png = Png::from_bytes(input.into()).expect("unable to get as png");

            let mut output_png = Png::from_bytes(output.into()).expect("unable to get as png");
            output_png.set_exif(input_png.exif());
            output_png.set_icc_profile(input_png.icc_profile());

            let output_file = OpenOptions::new()
                .write(true)
                .open(to)
                .expect("unable to open png as File");
            output_png
                .encoder()
                .write_to(&output_file)
                .expect("cannot write to output png file");
        }
        "jpg" | "jpeg" => {
            let input_jpg = Jpeg::from_bytes(input.into()).expect("unable to get as jpeg");

            let mut output_jpg = Jpeg::from_bytes(output.into()).expect("unable to get as jpeg");
            output_jpg.set_exif(input_jpg.exif());
            output_jpg.set_icc_profile(input_jpg.icc_profile());

            let output_file = OpenOptions::new()
                .write(true)
                .open(to)
                .expect("unable to open jpeg as File");
            output_jpg
                .encoder()
                .write_to(&output_file)
                .expect("cannot write to output jpg file");
        }
        other => error!("Extension ({other}) not supported to get Exif metadata: {from:?}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::recopy_metadata;
    use img_parts::jpeg::Jpeg;
    use img_parts::ImageEXIF;

    #[test]
    fn test_exif_read() {
        let input = std::fs::read("data/exif.jpg").unwrap();
        let jpg = Jpeg::from_bytes(input.into()).unwrap();
        let exif = jpg.exif().unwrap();
        // println!("{:?}", exif);
        // TODO better test
        assert!(exif.contains(&b'V'));
        assert!(exif.contains(&b'C'));
        assert!(exif.contains(&b'Y'));
        assert!(exif.contains(&b'B'));
        // assert!(exif.contains("V\0I\0L\0L\0A\0C\0O\0U\0B\0L\0A\0Y\0 \0B\0A\0T\0A\0I\0L\0L\0O\0N\0 \0A\0I\0R\0 \01\0.\01\00\07\0 \01\09\04\05\0.\04\06\0\0\0".as_bytes()))
    }

    #[test]
    fn test_exif_write() {
        let input = "data/exif.jpg";
        // let with_image = "data/original.jpg";
        let work = "data/exif_work.jpg";
        std::fs::copy(input, work).unwrap();
        recopy_metadata(input, work).unwrap();

        let output_raw = std::fs::read(work).unwrap();
        let jpg = Jpeg::from_bytes(output_raw.into()).unwrap();
        let exif = jpg.exif().unwrap();
        // println!("{:?}", exif);
        // TODO better test
        assert!(exif.contains(&b'V'));
        assert!(exif.contains(&b'C'));
        assert!(exif.contains(&b'Y'));
        assert!(exif.contains(&b'B'));
        std::fs::remove_file(work).unwrap();
    }
}
