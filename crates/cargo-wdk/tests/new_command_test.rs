//! System level tests for cargo wdk new flow
use assert_cmd::Command;
use assert_fs::TempDir;

#[test]
fn given_a_cargo_wdk_new_command_when_driver_type_is_kmdf_then_it_creates_valid_umdf_driver_project(
) {
    create_add_assert_new_driver_proj("kmdf");
}

#[test]
fn given_a_cargo_wdk_new_command_when_driver_type_is_umdf_then_it_creates_valid_umdf_driver_project(
) {
    create_add_assert_new_driver_proj("umdf");
}

#[test]
fn given_a_cargo_wdk_new_command_when_driver_type_is_wdm_then_it_creates_valid_umdf_driver_project()
{
    create_add_assert_new_driver_proj("wdm");
}

fn create_add_assert_new_driver_proj(driver_type: &str) {
    let driver_name = format!("test-{}-driver", driver_type);
    let driver_name_underscored = driver_name.replace("-", "_");
    let tmp_dir = TempDir::new().unwrap();
    println!("Temp dir: {}", tmp_dir.path().display());
    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "new",
        &driver_name,
        "kmdf",
        "--cwd",
        &tmp_dir.to_string_lossy(), // Root dir for tests is cargo-wdk
    ]);

    // assert command output
    let cmd_assertion = cmd.assert().success();
    let output = cmd_assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("stdout: {}", stdout);
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!(
        "driver path: {}",
        tmp_dir.path().join(&driver_name_underscored).display()
    );
    assert!(stdout.contains(&format!(
        "New Driver Project {} created at {}",
        &driver_name_underscored,
        tmp_dir.join(&driver_name).display()
    )));

    assert!(tmp_dir.join(&driver_name).exists());
    assert!(tmp_dir.join(&driver_name).join("build.rs").exists());
    assert!(tmp_dir.join(&driver_name).join("Cargo.toml").exists());
    assert!(tmp_dir
        .join(&driver_name)
        .join(format!("{driver_name_underscored}.inx"))
        .exists());
    assert!(tmp_dir
        .join(&driver_name)
        .join("src")
        .join("lib.rs")
        .exists());

    let mut cmd = Command::cargo_bin("cargo-wdk").expect("unable to find cargo-wdk binary");
    cmd.args([
        "build",
        "--cwd",
        &tmp_dir.to_string_lossy(), // Root dir for tests is cargo-wdk
    ]);

    cmd.assert().success();
    tmp_dir.close().unwrap();
}
