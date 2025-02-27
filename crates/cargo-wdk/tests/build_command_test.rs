//! Integration tests for package flow
use std::{path::PathBuf, process::Command};

use assert_cmd::prelude::*;

#[test]
fn given_a_mixed_package_kmdf_workspace_when_cargo_wdk_is_executed_then_driver_package_folder_is_created_with_expected_files(
) {
    if let Ok(rustflags) = std::env::var("RUSTFLAGS") {
        let updated_rust_flags = format!("{rustflags} -C target-feature=+crt-static");
        std::env::set_var("RUSTFLAGS", updated_rust_flags);
        println!("RUSTFLAGS set, adding the +crt-static: {rustflags:?}");
    } else {
        std::env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
        println!(
            "No RUSTFLAGS set, setting it to: {:?}",
            std::env::var("RUSTFLAGS").expect("RUSTFLAGS not set")
        );
    }
    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "build",
        "--cwd",
        "tests/mixed-package-kmdf-workspace", // Root dir for tests is cargo-wdk
    ]);

    // assert command output
    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Processing completed for package: driver"));
    assert!(stdout.contains(
        "No package.metadata.wdk section found. Skipping driver package workflow for package: \
         non_driver_crate"
    ));

    // assert driver package
    assert!(
        PathBuf::from("tests/mixed-package-kmdf-workspace/target/debug/driver_package").exists()
    );
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.cat"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.inf"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.map"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.pdb"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.sys"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/WDRLocalTestCert.cer"
    )
    .exists());
}

#[test]
fn given_a_umdf_driver_when_cargo_wdk_is_executed_then_driver_package_folder_is_created_with_expected_files(
) {
    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "build",
        "--cwd",
        "tests/umdf-driver", // Root dir for tests is cargo-wdk
    ]);

    // assert command output
    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Processing completed for package: umdf-driver"));

    // assert driver package
    assert!(PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package").exists());
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.cat")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.inf")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.map")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.pdb")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.dll")
            .exists()
    );
    assert!(PathBuf::from(
        "tests/umdf-driver/target/debug/umdf_driver_package/WDRLocalTestCert.cer"
    )
    .exists());
}

#[test]
fn given_an_emulated_workspace_when_cargo_wdk_is_executed_then_all_driver_projects_are_built_and_packaged_and_expected_files_are_created_in_package_folders(
) {
    if let Ok(rustflags) = std::env::var("RUSTFLAGS") {
        let updated_rust_flags = format!("{rustflags} -C target-feature=+crt-static");
        std::env::set_var("RUSTFLAGS", updated_rust_flags);
        println!("RUSTFLAGS set, adding the +crt-static: {rustflags:?}");
    } else {
        std::env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
        println!(
            "No RUSTFLAGS set, setting it to: {:?}",
            std::env::var("RUSTFLAGS").expect("RUSTFLAGS not set")
        );
    }

    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "build", "--cwd", "tests", // Tests itself can be viewed as emulated workspace
    ]);

    // assert command output
    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Processing completed for package: driver"));
    assert!(stdout.contains(
        "No package.metadata.wdk section found. Skipping driver package workflow for package: \
         non_driver_crate"
    ));

    assert!(stdout.contains("Processing completed for package: umdf-driver"));

    // assert mixed-package-kmdf-workspace driver package
    assert!(
        PathBuf::from("tests/mixed-package-kmdf-workspace/target/debug/driver_package").exists()
    );
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.cat"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.inf"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.map"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.pdb"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.sys"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/mixed-package-kmdf-workspace/target/debug/driver_package/WDRLocalTestCert.cer"
    )
    .exists());

    // assert umdf-driver package
    assert!(PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package").exists());
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.cat")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.inf")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.map")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.pdb")
            .exists()
    );
    assert!(
        PathBuf::from("tests/umdf-driver/target/debug/umdf_driver_package/umdf_driver.dll")
            .exists()
    );
    assert!(PathBuf::from(
        "tests/umdf-driver/target/debug/umdf_driver_package/WDRLocalTestCert.cer"
    )
    .exists());
}
