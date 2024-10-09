use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatalogError {
    #[error("No .cab or invc.exe files found: {0}")]
    NoFilesFound(String),

    #[error("Multiple .cab and invc.exe files found: {0}\n{1}")]
    MultipleFilesFound(String, String),

    #[error(transparent)]
    IoError(#[from] io::Error),
}
