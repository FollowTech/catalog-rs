use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatalogError {
    #[error("No .cab file and invc.exe file found")]
    SelectedIcFileError,
    #[error("No .cab file and invc.exe file found")]
    SelectedCabFileError,
    #[error("No .cab file and invc.exe file found")]
    NoFilesFound,

    #[error("Multiple .cab and invc.exe files found")]
    MultipleFilesFound,

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
}
