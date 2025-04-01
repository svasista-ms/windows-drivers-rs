//! System level tests for cargo wdk build flow
mod common;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use assert_cmd::prelude::*;
use common::set_crt_static_flag;
use serial_test::serial;
use sha256::try_digest;

#[test]
#[serial]
fn given_a_mixed_package_kmdf_workspace_when_cargo_wdk_is_executed_then_driver_package_folder_is_created_with_expected_files(
) {
    set_crt_static_flag();

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

    // assert if the files are copied properly
    assert_eq!(
        try_digest(Path::new(
            "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.map"
        ))
        .expect("Unable to read packaged driver.map"),
        try_digest(Path::new(
            "tests/mixed-package-kmdf-workspace/target/debug/deps/driver.map"
        ))
        .expect("Unable to read source driver.map")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/mixed-package-kmdf-workspace/target/debug/driver_package/driver.pdb"
        ))
        .expect("Unable to read packaged driver.pdb"),
        try_digest(Path::new(
            "tests/mixed-package-kmdf-workspace/target/debug/driver.pdb"
        ))
        .expect("Unable to read source driver.pdb")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/mixed-package-kmdf-workspace/target/debug/driver_package/WDRLocalTestCert.cer"
        ))
        .expect("Unable to read packaged WDRLocalTestCert.cer"),
        try_digest(Path::new(
            "tests/mixed-package-kmdf-workspace/target/debug/WDRLocalTestCert.cer"
        ))
        .expect("Unable to read source WDRLocalTestCert.cer")
    );
}

#[test]
#[serial]
fn given_a_kmdf_driver_when_cargo_wdk_is_executed_then_driver_package_folder_is_created_with_expected_files(
) {
    build_driver_project("kmdf");
}

#[test]
#[serial]
fn given_a_umdf_driver_when_cargo_wdk_is_executed_then_driver_package_folder_is_created_with_expected_files(
) {
    build_driver_project("umdf");
}

#[test]
#[serial]
fn given_a_wdm_driver_when_cargo_wdk_is_executed_then_driver_package_folder_is_created_with_expected_files(
) {
    build_driver_project("wdm");
}

fn build_driver_project(driver_type: &str) {
    set_crt_static_flag();

    let driver_folder = format!("{driver_type}-driver");
    let driver_folder_underscored = format!("{driver_type}_driver");

    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "build",
        "--cwd",
        &format!("tests/{driver_folder}"), // Root dir for tests is cargo-wdk
    ]);

    // assert command output
    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!(
        "Processing completed for package: {driver_folder}"
    )));

    // assert driver package
    assert!(PathBuf::from(format!(
        "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package"
    ))
    .exists());
    assert!(PathBuf::from(format!(
        "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
         {driver_folder_underscored}.cat"
    ))
    .exists());
    assert!(PathBuf::from(format!(
        "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
         {driver_folder_underscored}.inf"
    ))
    .exists());
    assert!(PathBuf::from(format!(
        "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
         {driver_folder_underscored}.map"
    ))
    .exists());
    assert!(PathBuf::from(format!(
        "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
         {driver_folder_underscored}.pdb"
    ))
    .exists());

    if matches!(driver_type, "kmdf" | "wdm") {
        assert!(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
             {driver_folder_underscored}.sys"
        ))
        .exists());
    } else {
        assert!(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
             {driver_folder_underscored}.dll"
        ))
        .exists());
    }

    assert!(PathBuf::from(format!(
        "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/WDRLocalTestCert.\
         cer"
    ))
    .exists());

    // assert if the files are copied properly
    assert_eq!(
        try_digest(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
             {driver_folder_underscored}.map"
        )))
        .unwrap_or_else(|_| format!("Unable to read packaged {driver_folder_underscored}.map")),
        try_digest(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/deps/{driver_folder_underscored}.map"
        )))
        .unwrap_or_else(|_| format!("Unable to read source {driver_folder_underscored}.map")),
    );

    assert_eq!(
        try_digest(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
             {driver_folder_underscored}.pdb"
        )))
        .unwrap_or_else(|_| format!("Unable to read packaged {driver_folder_underscored}.pdb")),
        try_digest(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/{driver_folder_underscored}.pdb"
        )))
        .unwrap_or_else(|_| format!("Unable to read source {driver_folder_underscored}.pdb"))
    );

    assert_eq!(
        try_digest(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/{driver_folder_underscored}_package/\
             WDRLocalTestCert.cer"
        )))
        .unwrap_or_else(|_| "Unable to read packaged WDRLocalTestCert.cer".to_string()),
        try_digest(PathBuf::from(format!(
            "tests/{driver_folder}/target/debug/WDRLocalTestCert.cer"
        )))
        .unwrap_or_else(|_| "Unable to read source WDRLocalTestCert.cer".to_string())
    );
}

#[test]
#[serial]
fn given_an_emulated_workspace_when_cargo_wdk_is_executed_then_all_driver_projects_are_built_and_packaged_and_non_driver_rust_projects_failed_and_rest_ignored(
) {
    set_crt_static_flag();

    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "build",
        "--cwd",
        "tests/emulated-workspace", // Root dir for tests is cargo-wdk
    ]);

    // assert command output
    let cmd_assertion = cmd.assert().failure(); // Since setup includes non driver rust project
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(
        "Error packaging the child project: rust-project, error: Error Parsing WDK metadata from \
         Cargo.toml"
    )); // rust-project is not a driver and it is expected to fail
    assert!(stdout.contains("Processing completed for package: driver_1"));
    assert!(stdout.contains("Processing completed for package: driver_2"));
    assert!(stdout.contains(
        r"One or more rust (possibly driver) projects failed to package in the working directory: "
    ));

    // assert umdf-driver-workspace driver package
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/driver_1.cat"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/driver_1.inf"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/driver_1.map"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/driver_1.pdb"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/driver_1.dll"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/\
         WDRLocalTestCert.cer"
    )
    .exists());

    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/driver_2.cat"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/driver_2.inf"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/driver_2.map"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/driver_2.pdb"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/driver_2.dll"
    )
    .exists());
    assert!(PathBuf::from(
        "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/\
         WDRLocalTestCert.cer"
    )
    .exists());

    // assert if the files are copied properly
    assert_eq!(
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/\
             driver_1.map"
        ))
        .expect("Unable to read packaged driver_1.map"),
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/deps/driver_1.map"
        ))
        .expect("Unable to read source driver_1.map")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/\
             driver_1.pdb"
        ))
        .expect("Unable to read packaged driver_1.pdb"),
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1.pdb"
        ))
        .expect("Unable to read source driver_1.pdb")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_1_package/\
             WDRLocalTestCert.cer"
        ))
        .expect("Unable to read packaged WDRLocalTestCert.cer"),
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/WDRLocalTestCert.cer"
        ))
        .expect("Unable to read source WDRLocalTestCert.cer")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/\
             driver_2.map"
        ))
        .expect("Unable to read packaged driver_2.map"),
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/deps/driver_2.map"
        ))
        .expect("Unable to read source driver_2.map")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/\
             driver_2.pdb"
        ))
        .expect("Unable to read packaged driver_2.pdb"),
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2.pdb"
        ))
        .expect("Unable to read source driver_2.pdb")
    );

    assert_eq!(
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/driver_2_package/\
             WDRLocalTestCert.cer"
        ))
        .expect("Unable to read packaged WDRLocalTestCert.cer"),
        try_digest(Path::new(
            "tests/emulated-workspace/umdf-driver-workspace/target/debug/WDRLocalTestCert.cer"
        ))
        .expect("Unable to read source WDRLocalTestCert.cer")
    );
}
