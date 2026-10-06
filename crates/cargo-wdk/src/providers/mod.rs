// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0
//! The `providers` module serves as a centralized abstraction layer for various
//! subsystems used throughout the application. It encapsulates functionality
//! such as file system operations, command execution,
//! metadata handling, and interactions with the `wdk-build` crate. By
//! consolidating these external dependencies, the module promotes cleaner
//! separation of concerns and enhances testability. This design allows external
//! calls to be easily mocked, simplifying unit testing and enabling more robust
//! and maintainable code in the action layer.

pub mod exec;
pub mod fs;
pub mod metadata;
pub mod wdk_build;

use std::ffi::OsString;

use self::error::CargoPathError;

/// Returns the Cargo executable supplied by the invoking environment.
///
/// # Errors
/// Returns an error if `CARGO` is missing or empty.
pub fn cargo_path() -> Result<OsString, CargoPathError> {
    cargo_path_from_env(std::env::var_os("CARGO"))
}

fn cargo_path_from_env(value: Option<OsString>) -> Result<OsString, CargoPathError> {
    match value {
        None => Err(CargoPathError::Missing),
        Some(path) if path.is_empty() => Err(CargoPathError::Empty),
        Some(path) => Ok(path),
    }
}

pub mod error {
    use std::{io, path::PathBuf, process::Output};

    /// Errors validating the Cargo executable supplied by the environment.
    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    pub enum CargoPathError {
        #[error("CARGO is not set; invoke through `cargo wdk ...`")]
        Missing,
        #[error("CARGO is empty")]
        Empty,
    }

    /// Errors resolving Cargo or retrieving project metadata.
    #[derive(Debug, thiserror::Error)]
    pub enum MetadataError {
        #[error(transparent)]
        CargoPath(#[from] CargoPathError),
        #[error(transparent)]
        CargoMetadata(#[from] cargo_metadata::Error),
    }

    /// Error type for `std::process::command` execution failures
    #[derive(Debug, thiserror::Error)]
    pub enum CommandError {
        #[error("Command '{command}' with args {args:?} failed \n STDOUT: {stdout}")]
        CommandFailed {
            command: String,
            args: Vec<String>,
            stdout: String,
        },
        #[error("Command '{0}' with args {1:?} IO error")]
        IoError(String, Vec<String>, #[source] io::Error),
    }

    impl CommandError {
        pub fn from_output(command: &str, args: &[&str], output: &Output) -> Self {
            Self::CommandFailed {
                command: command.to_string(),
                args: args.iter().map(|&s| s.to_string()).collect(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            }
        }

        pub fn from_io_error(command: &str, args: &[&str], e: io::Error) -> Self {
            Self::IoError(
                command.to_string(),
                args.iter().map(|&s| s.to_string()).collect(),
                e,
            )
        }
    }

    /// Error type for `std::file` operations
    #[derive(Debug, thiserror::Error)]
    pub enum FileError {
        #[error("File {0} not found")]
        NotFound(PathBuf),
        #[error("Failed to write to file {0}")]
        WriteError(PathBuf, #[source] io::Error),
        #[error("Failed to read file {0}")]
        ReadError(PathBuf, #[source] io::Error),
        #[error("Failed to open file {0}")]
        OpenError(PathBuf, #[source] io::Error),
        #[error("Failed to append to file {0}")]
        AppendError(PathBuf, #[source] io::Error),
        #[error("Failed to copy file from {0} to {1}")]
        CopyError(PathBuf, PathBuf, #[source] io::Error),
        #[error("Failed to create directory at path {0}")]
        CreateDirError(PathBuf, #[source] io::Error),
        #[error("Failed to rename file from {0} to {1}")]
        RenameError(PathBuf, PathBuf, #[source] io::Error),
        #[error("Failed to remove directory {0}")]
        RemoveDirError(PathBuf, #[source] io::Error),
        #[error("Failed to get file type for directory entry {0}")]
        DirFileTypeError(PathBuf, #[source] io::Error),
        #[error("Failed to read directory {0}")]
        ReadDirError(PathBuf, #[source] io::Error),
        #[error("Failed to read directory entries for {0}")]
        ReadDirEntriesError(PathBuf, #[source] io::Error),
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    use super::{cargo_path_from_env, error::CargoPathError};

    #[test]
    fn missing_cargo_is_an_error() {
        let error = cargo_path_from_env(None).unwrap_err();
        assert_eq!(error, CargoPathError::Missing);
        assert_eq!(
            error.to_string(),
            "CARGO is not set; invoke through `cargo wdk ...`"
        );
    }

    #[test]
    fn empty_cargo_is_an_error() {
        let error = cargo_path_from_env(Some(OsString::new())).unwrap_err();
        assert_eq!(error, CargoPathError::Empty);
        assert_eq!(error.to_string(), "CARGO is empty");
    }

    #[test]
    fn non_unicode_cargo_path_is_preserved() {
        let path = OsString::from_wide(&[0x0043, 0x003A, 0x005C, 0xD800]);
        assert_eq!(cargo_path_from_env(Some(path.clone())).unwrap(), path);
    }

    #[test]
    fn cargo_path_is_preserved() {
        let path = r"C:\nonexistent cargo-wdk test toolchain\bin\cargo.exe";
        assert!(!std::path::Path::new(path).exists());
        assert_eq!(cargo_path_from_env(Some(path.into())).unwrap(), path);
    }
}
