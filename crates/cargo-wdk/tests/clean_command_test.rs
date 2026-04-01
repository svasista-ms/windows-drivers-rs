//! Integration tests for the `cargo wdk clean` command.
mod test_utils;

use assert_cmd::prelude::*;
use test_utils::{create_cargo_wdk_cmd, with_mutex};

fn run_clean_cmd(path: &str) -> String {
    let mut cmd = create_cargo_wdk_cmd("clean", None, None, Some(path));
    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn standalone_project_cleans_successfully() {
    let project_path = "tests/umdf-driver";
    with_mutex(project_path, || {
        run_clean_cmd(project_path);
    });
}

#[test]
fn workspace_cleans_successfully() {
    let workspace_path = "tests/emulated-workspace/umdf-driver-workspace";
    with_mutex(workspace_path, || {
        run_clean_cmd(workspace_path);
    });
}

#[test]
fn emulated_workspace_cleans_successfully() {
    let emulated_workspace_path = "tests/emulated-workspace";
    with_mutex(emulated_workspace_path, || {
        run_clean_cmd(emulated_workspace_path);
    });
}
