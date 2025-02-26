use thiserror::Error;

use crate::providers::error::{CommandError, FileError};

#[derive(Debug, Error)]
pub enum NewProjectError {
    #[error("Error creating new driver project, error: {0}")]
    NewDriver(NewDriverError),
}

#[derive(Debug, Error)]
pub enum NewDriverError {
    #[error("Error executing cargo new, error: {0}")]
    CargoNewCommand(CommandError),
    #[error("File System Error, error: {0}")]
    FileSystem(#[from] FileError),
    #[error("Template file not found: {0}")]
    TemplateNotFound(String),
    #[error("IO Error")]
    Io(#[from] std::io::Error),
}
