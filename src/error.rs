use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ProcessError {
    #[error("Filesystem error")]
    FsError(String),
}
