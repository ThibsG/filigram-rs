use log::debug;
use std::path::Path;

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
    pub fn is_file_qualified(&self, path: &impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        if let Some(extension) = path.extension() {
            let extension = extension
                .to_str()
                .expect("can't convert to str")
                .to_lowercase();

            if !self
                .authorized_extensions
                .iter()
                .any(|ext| ext.as_str() == extension)
            {
                debug!("file ignored (bad extension): {path:?}");
                return false;
            }
        } else {
            debug!("file ignored (no extension): {path:?}");
            return false;
        }

        let path_str = path
            .file_name()
            .expect("can't retrieve filename")
            .to_str()
            .expect("unable to convert filename to str");

        if self
            .excluded_files
            .iter()
            .any(|excluded_filename| path_str.starts_with(excluded_filename))
        {
            debug!("file ignored (excluded file): {path:?}");
            return false;
        }

        if self.excluded_dirs.iter().any(|dir| {
            path.components().any(|comp| {
                comp.as_os_str().to_str().expect("can't convert an OsStr") == dir.as_str()
            })
        }) {
            debug!("file ignored (dir excluded): {path:?}");
            return false;
        }

        true
    }
}
