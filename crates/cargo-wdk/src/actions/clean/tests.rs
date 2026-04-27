// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

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
