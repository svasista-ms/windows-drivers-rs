// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0
//! This module contains the `CleanAction` struct and its associated methods
//! for cleaning build artifacts produced by the `build` command.
mod error;

use std::path::{Path, PathBuf, absolute};

use anyhow::Result;
use error::CleanActionError;
use mockall_double::double;
use tracing::{debug, error as err, info};

#[double]
use crate::providers::{exec::CommandExec, fs::Fs};
use crate::trace;

/// Action that removes build artifacts produced by the `build` command for a
/// driver project or emulated workspace.
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
        anyhow::ensure!(
            !working_dir.as_os_str().is_empty(),
            "working_dir must not be empty"
        );
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

            if !found_at_least_one_project {
                info!("Cleaning package(s) in {}", self.working_dir.display());
            }
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

#[cfg(test)]
mod tests {
    use std::{
        os::windows::process::ExitStatusExt,
        path::PathBuf,
        process::{ExitStatus, Output},
    };

    use mockall::predicate::eq;
    use mockall_double::double;

    use super::{CleanAction, error::CleanActionError};
    use crate::providers::error::CommandError;
    #[double]
    use crate::providers::{exec::CommandExec, fs::Fs};

    fn create_successful_output() -> Output {
        Output {
            status: ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn new_succeeds_for_valid_args() {
        let cwd = PathBuf::from("C:\\tmp");
        let mock_fs = Fs::default();
        let mock_exec = CommandExec::default();

        let action = CleanAction::new(
            &cwd,
            clap_verbosity_flag::Verbosity::default(),
            &mock_exec,
            &mock_fs,
        );

        assert!(action.is_ok());
    }

    #[test]
    fn new_fails_if_working_dir_is_empty() {
        let cwd = PathBuf::from("");
        let mock_fs = Fs::default();
        let mock_exec = CommandExec::default();

        let action = CleanAction::new(
            &cwd,
            clap_verbosity_flag::Verbosity::default(),
            &mock_exec,
            &mock_fs,
        );

        let err = action
            .err()
            .expect("CleanAction::new should fail for empty working_dir");
        assert_eq!(err.to_string(), "working_dir must not be empty");
    }

    #[test]
    fn run_invokes_cargo_clean_and_succeeds() {
        let cwd = PathBuf::from("C:\\tmp");
        let mut mock_fs = Fs::default();
        let mut mock_exec = CommandExec::default();

        mock_fs
            .expect_exists()
            .with(eq(cwd.join("Cargo.toml")))
            .returning(|_| true);

        mock_exec
            .expect_run()
            .withf(move |cmd, args, _env, working_dir| {
                cmd == "cargo"
                    && args == ["clean"]
                    && *working_dir == Some(PathBuf::from("C:\\tmp").as_path())
            })
            .returning(|_, _, _, _| Ok(create_successful_output()));

        let action = CleanAction::new(
            &cwd,
            clap_verbosity_flag::Verbosity::default(),
            &mock_exec,
            &mock_fs,
        )
        .expect("CleanAction::new should succeed");

        assert!(action.run().is_ok());
    }

    #[test]
    fn run_returns_error_when_cargo_clean_fails() {
        let cwd = PathBuf::from("C:\\tmp");
        let mut mock_fs = Fs::default();
        let mut mock_exec = CommandExec::default();

        mock_fs
            .expect_exists()
            .with(eq(cwd.join("Cargo.toml")))
            .returning(|_| true);

        mock_exec
            .expect_run()
            .withf(move |cmd, args, _env, working_dir| {
                cmd == "cargo"
                    && args == ["clean"]
                    && *working_dir == Some(PathBuf::from("C:\\tmp").as_path())
            })
            .returning(|_, _, _, _| {
                Err(CommandError::CommandFailed {
                    command: "cargo".to_string(),
                    args: vec!["clean".to_string()],
                    stdout: "error".to_string(),
                })
            });

        let action = CleanAction::new(
            &cwd,
            clap_verbosity_flag::Verbosity::default(),
            &mock_exec,
            &mock_fs,
        )
        .expect("CleanAction::new should succeed");

        let result = action.run();
        assert!(matches!(result, Err(CleanActionError::CargoClean(_))));
    }

    #[test]
    fn run_returns_error_when_no_cargo_toml_and_no_rust_projects_are_found() {
        let cwd = PathBuf::from("C:\\tmp");
        let mut mock_fs = Fs::default();
        let mock_exec = CommandExec::default();

        mock_fs
            .expect_exists()
            .with(eq(cwd.join("Cargo.toml")))
            .returning(|_| false);

        mock_fs.expect_read_dir_entries().returning(|_| Ok(vec![]));

        let action = CleanAction::new(
            &cwd,
            clap_verbosity_flag::Verbosity::default(),
            &mock_exec,
            &mock_fs,
        )
        .expect("CleanAction::new should succeed");

        let result = action.run();
        assert!(matches!(
            result,
            Err(CleanActionError::NoValidRustProjectsInTheDirectory(_))
        ));
    }
}
