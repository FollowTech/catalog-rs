use std::{fmt::Display, io};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatalogError {
    #[error("No .cab or invc.exe files found: {0}")]
    NoFilesFound(String),

    #[error("Multiple .cab and invc.exe files found: {0}\n{1}")]
    MultipleFilesFound(String, String),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("Failed to parse catalog: {0}")]
    ParseError(String),

    #[error("Failed to parse catalog: {0}")]
    InvalidCatalog(String),

    #[error(transparent)]
    IcedError(#[from] iced::Error),
}
