//! System level tests for `--help` output of cargo-wdk commands.
use assert_cmd::{Command, assert::OutputAssertExt};

fn run_help(args: &[&str]) -> (String, String) {
    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args(args);

    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    println!("stdout: {stdout}");
    println!("stderr: {stderr}");
    (stdout, stderr)
}

#[test]
fn top_level_help_lists_subcommands_and_options() {
    let (stdout, stderr) = run_help(&["--help"]);

    assert!(stderr.is_empty());
    // About string
    assert!(stdout.contains(
        "cargo-wdk is a cargo extension that can be used to create and build Windows Rust driver \
         projects."
    ));
    // Usage line uses the configured bin_name
    assert!(stdout.contains("Usage: cargo wdk"));
    // Subcommands are advertised
    assert!(stdout.contains("new"));
    assert!(stdout.contains("Create a new Windows Driver Kit project"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("Build the Windows Driver Kit project"));
    assert!(stdout.contains("help"));
    // Global options
    assert!(stdout.contains("-h, --help"));
    assert!(stdout.contains("-V, --version"));
    // Verbosity flags from clap-verbosity-flag
    assert!(stdout.contains("Verbosity"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--quiet"));
}

#[test]
fn new_help_advertises_all_options() {
    let (stdout, stderr) = run_help(&["new", "--help"]);

    assert!(stderr.is_empty());
    assert!(stdout.contains("Create a new Windows Driver Kit project"));
    assert!(stdout.contains("Usage: cargo wdk new [OPTIONS] <--kmdf|--umdf|--wdm> <PATH>"));
    // Positional
    assert!(stdout.contains("<PATH>"));
    assert!(stdout.contains("Path at which the new driver crate should be created"));
    // Driver-type flags (in mutually-exclusive group)
    assert!(stdout.contains("--kmdf"));
    assert!(stdout.contains("Create a KMDF driver crate"));
    assert!(stdout.contains("--umdf"));
    assert!(stdout.contains("Create a UMDF driver crate"));
    assert!(stdout.contains("--wdm"));
    assert!(stdout.contains("Create a WDM driver crate"));
    // Help flag
    assert!(stdout.contains("-h, --help"));
}

#[test]
fn build_help_advertises_all_options() {
    let (stdout, stderr) = run_help(&["build", "--help"]);

    assert!(stderr.is_empty());
    assert!(stdout.contains("Build the Windows Driver Kit project"));
    assert!(stdout.contains("Usage: cargo wdk build [OPTIONS]"));
    // Build flags
    assert!(stdout.contains("--profile"));
    assert!(stdout.contains("Build artifacts with the specified profile"));
    assert!(stdout.contains("--target-arch"));
    assert!(stdout.contains("Build for the target architecture"));
    assert!(stdout.contains("--verify-signature"));
    assert!(stdout.contains("Verify the signature"));
    assert!(stdout.contains("--sample"));
    assert!(stdout.contains("Build sample class driver project"));
    // Help flag
    assert!(stdout.contains("-h, --help"));
}
