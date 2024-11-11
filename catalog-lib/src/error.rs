use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatalogError {
    #[error("{0}")]
    SelectedFileError(String),

    #[error("No {0} file found")]
    CurrentFileError(String),

    #[error("Unexpected error")]
    Unexpected,

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("Failed to parse catalog: {0}")]
    ParseError(String),

    #[error("Failed to parse catalog: {0}")]
    InvalidCatalog(String),

    #[error(transparent)]
    IcedError(#[from] iced::Error),

    #[error(transparent)]
    WindsowsError(#[from] windows::core::Error),
}
