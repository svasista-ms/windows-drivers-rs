// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0
//! This module contains the `CleanAction` struct and its associated methods
//! for cleaning build artifacts produced by the `build` command.
mod error;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf, absolute};

use anyhow::Result;
use error::CleanActionError;
use mockall_double::double;
use tracing::{debug, error as err, info};

#[double]
use crate::providers::{exec::CommandExec, fs::Fs};
use crate::trace;

pub struct CleanAction<'a> {
    working_dir: PathBuf,
    verbosity_level: clap_verbosity_flag::Verbosity,

    // Injected deps
    command_exec: &'a CommandExec,
    fs: &'a Fs,
}

impl<'a> CleanAction<'a> {
    /// Creates a new instance of `CleanAction`.
    ///
    /// # Arguments
    /// * `working_dir` - The working directory for the clean action
    /// * `verbosity_level` - The verbosity level for logging
    /// * `command_exec` - The command execution provider instance
    /// * `fs` - The file system provider instance
    ///
    /// # Returns
    /// * `Result<Self>` - A result containing either a new instance of
    ///   `CleanAction` on success, or an `anyhow::Error`.
    ///
    /// # Errors
    /// * [`anyhow::Error`] - If `working_dir` is not a syntactically valid
    ///   path, e.g. it is empty
    pub fn new(
        working_dir: &Path,
        verbosity_level: clap_verbosity_flag::Verbosity,
        command_exec: &'a CommandExec,
        fs: &'a Fs,
    ) -> Result<Self> {
        Ok(Self {
            working_dir: absolute(working_dir)?,
            verbosity_level,
            command_exec,
            fs,
        })
    }

    /// Entry point method to execute the clean action flow.
    ///
    /// The detection strategy is:
    /// 1. If the working directory has a `Cargo.toml`, run `cargo clean`
    ///    directly (standalone project or workspace root).
    /// 2. Otherwise, treat the directory as an emulated workspace: scan
    ///    immediate subdirectories for Rust projects and clean each. NOTE: This
    ///    follows the same logic as the build action.
    ///
    /// # Returns
    /// `Result<(), CleanActionError>`
    ///
    /// # Errors
    /// * `CleanActionError::FileIo` - If there is an IO error.
    /// * `CleanActionError::CargoClean` - If there is an error running the
    ///   `cargo clean` command.
    /// * `CleanActionError::NoValidRustProjectsInTheDirectory` - If no valid
    ///   Rust projects are found in the working directory.
    /// * `CleanActionError::OneOrMoreRustProjectsFailedToClean` - If one or
    ///   more Rust projects fail to clean in an emulated workspace.
    pub fn run(&self) -> Result<(), CleanActionError> {
        debug!(
            "Attempting to clean project at: {}",
            self.working_dir.display()
        );

        // Standalone driver/driver workspace support
        if self.fs.exists(&self.working_dir.join("Cargo.toml")) {
            debug!(
                "Found Cargo.toml in {}. Running cargo clean.",
                self.working_dir.display()
            );
            return self.run_cargo_clean(&self.working_dir);
        }

        // Emulated workspaces support
        let dirs = self.fs.read_dir_entries(&self.working_dir)?;
        debug!(
            "Checking for valid Rust projects in the working directory: {}",
            self.working_dir.display()
        );

        info!("Cleaning package(s) in {}", self.working_dir.display());

        let mut found_at_least_one_project = false;
        let mut failed_at_least_one_project = false;
        for dir in dirs {
            debug!("Checking dir entry: {}", dir.path().display());
            if !self.fs.dir_file_type(&dir)?.is_dir()
                || !self.fs.exists(&dir.path().join("Cargo.toml"))
            {
                debug!("Dir entry is not a valid Rust package");
                continue;
            }

            let working_dir_path = dir.path();
            let sub_dir = dir.file_name();
            let sub_dir = sub_dir.to_string_lossy();

            found_at_least_one_project = true;
            debug!("Cleaning package(s) in dir {sub_dir}");
            if let Err(e) = self.run_cargo_clean(&working_dir_path) {
                failed_at_least_one_project = true;
                err!(
                    "Error cleaning project: {sub_dir}, error: {:?}",
                    anyhow::Error::new(e)
                );
            }
        }

        if !found_at_least_one_project {
            return Err(CleanActionError::NoValidRustProjectsInTheDirectory(
                self.working_dir.clone(),
            ));
        }

        debug!("Done cleaning package(s) in {}", self.working_dir.display());
        if failed_at_least_one_project {
            return Err(CleanActionError::OneOrMoreRustProjectsFailedToClean(
                self.working_dir.clone(),
            ));
        }

        info!(
            "Clean completed successfully for package(s) in {}",
            self.working_dir.display()
        );
        Ok(())
    }

    /// Runs `cargo clean` in the specified directory.
    fn run_cargo_clean(&self, working_dir: &Path) -> Result<(), CleanActionError> {
        info!("Running cargo clean in {}", working_dir.display());
        let mut args = vec!["clean"];
        if let Some(flag) = trace::get_cargo_verbose_flags(self.verbosity_level) {
            args.push(flag);
        }
        self.command_exec
            .run("cargo", &args, None, Some(working_dir))
            .map_err(CleanActionError::CargoClean)?;
        info!("Cleaned project at {}", working_dir.display());
        Ok(())
    }
}
