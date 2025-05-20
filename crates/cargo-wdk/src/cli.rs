// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0
//! This module defines the top-level CLI layer, its argument types and
//! structures used for parsing and validating arguments for various
//! subcommands.
use std::path::PathBuf;

use anyhow::{Ok, Result};
use clap::{ArgGroup, Args, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use mockall_double::double;
use wdk_build::CpuArchitecture;

use crate::actions::{
    build::{BuildAction, BuildActionParams},
    new::NewAction,
    Profile,
    TargetArch,
};
#[double]
use crate::providers::{exec::CommandExec, fs::Fs, metadata::Metadata, wdk_build::WdkBuild};

const ABOUT_STRING: &str = "cargo-wdk is a cargo extension that can be used to create and build \
                            Windows Rust driver projects.";
const USAGE_STRING: &str = "cargo [+toolchain] wdk <subcommand> [options] [args]";
const CARGO_WDK_BIN_NAME: &str = "cargo wdk";

#[derive(Debug, Args)]
#[clap(
    group(
        ArgGroup::new("driver_type")
            .required(true)
            .args(&["kmdf", "umdf", "wdm"])
    ),
    override_usage = "cargo wdk new <--kmdf|--umdf|--wdm> <PATH> [OPTIONS]"
)]
pub struct NewCommandArgs {
    /// Create a KMDF driver crate
    #[clap(
        long,
        help = "Create a KMDF driver crate",
        help_heading = "Driver Configuration"
    )]
    pub kmdf: bool,

    /// Create a UMDF driver crate
    #[clap(
        long,
        help = "Create a UMDF driver crate",
        help_heading = "Driver Configuration"
    )]
    pub umdf: bool,

    /// Create a WDM driver crate
    #[clap(
        long,
        help = "Create a WDM driver crate",
        help_heading = "Driver Configuration"
    )]
    pub wdm: bool,

    /// Path at which the new crate should be created
    #[clap(
        value_name = "PATH",
        help = "Path at which the new crate should be created"
    )]
    pub path: PathBuf,
}

/// Arguments for the `build` subcommand
#[derive(Debug, Args)]
pub struct BuildProjectArgs {
    #[clap(long, help = "Path to the project", default_value = ".")]
    pub cwd: PathBuf,
    #[clap(long, help = "Build Profile/Configuration", ignore_case = true)]
    pub profile: Option<Profile>,
    #[clap(long, help = "Build Target", ignore_case = true)]
    pub target_arch: Option<CpuArchitecture>,
    #[clap(long, help = "Verify Signatures", default_value = "false")]
    pub verify_signature: bool,
    #[clap(
        long,
        help = "Build Sample Class Driver Project",
        default_value = "false"
    )]
    pub sample: bool,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Subcmd {
    #[clap(name = "new", about = "Create a new Windows Driver Kit project")]
    New(NewCommandArgs),
    #[clap(name = "build", about = "Build the Windows Driver Kit project")]
    Build(BuildProjectArgs),
}

/// Top level command line interface for cargo wdk
#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    bin_name = CARGO_WDK_BIN_NAME,
    display_name = CARGO_WDK_BIN_NAME,
    author = env!("CARGO_PKG_AUTHORS"),
    about = ABOUT_STRING,
    override_usage = USAGE_STRING,
)]
pub struct Cli {
    #[clap(name = "cargo command", default_value = "wdk", hide = true)]
    pub cargo_command: String,
    #[clap(subcommand)]
    pub sub_cmd: Subcmd,
    #[command(flatten)]
    #[clap(next_help_heading = "Verbosity")]
    pub verbose: Verbosity,
}

impl Cli {
    /// Entry point method to construct and call actions based on the subcommand
    /// and arguments provided by the user.
    pub fn run(self) -> Result<()> {
        let wdk_build = WdkBuild::default();
        let command_exec = CommandExec::default();
        let fs = Fs::default();
        let metadata = Metadata::default();

        match self.sub_cmd {
            Subcmd::New(cli_args) => {
                let driver_types = [
                    (cli_args.kmdf, "kmdf"),
                    (cli_args.umdf, "umdf"),
                    (cli_args.wdm, "wdm"),
                    // More driver types can be added here
                ];
                let selected: Vec<&str> = driver_types
                    .iter()
                    .filter_map(|(flag, name)| if *flag { Some(*name) } else { None })
                    .collect();
                if selected.len() != 1 {
                    return Err(anyhow::anyhow!(
                        "Please select exactly one driver type: kmdf, umdf, or wdm"
                    ));
                }
                let driver_type = selected[0];

                NewAction::new(
                    &cli_args.path,
                    driver_type,
                    self.verbose,
                    &command_exec,
                    &fs,
                )
                .run()?;

                Ok(())
            }
            Subcmd::Build(cli_args) => {
                let target_arch = if let Some(arch) = cli_args.target_arch {
                    TargetArch::Selected(arch)
                } else {
                    // Detect the default target architecture using rustc
                    let detected_arch =
                        Self::detect_default_target_arch_using_rustc(&command_exec)?;
                    TargetArch::Default(detected_arch)
                };
                let build_action = BuildAction::new(
                    &BuildActionParams {
                        working_dir: &cli_args.cwd,
                        profile: cli_args.profile.as_ref(),
                        target_arch,
                        verify_signature: cli_args.verify_signature,
                        is_sample_class: cli_args.sample,
                        verbosity_level: self.verbose,
                    },
                    &wdk_build,
                    &command_exec,
                    &fs,
                    &metadata,
                )?;
                build_action.run()?;
                Ok(())
            }
        }
    }

    /// Returns the default architecture of the host machine by running `rustc
    /// --print host-tuple` command.
    ///
    /// # Arguments
    /// * `command_exec` - A reference to the `CommandExec` struct that provides
    ///   methods for executing commands.
    /// # Returns
    /// * `CpuArchitecture`
    /// * `anyhow::Error` if the command fails to execute or the output is not
    ///   in the expected format.
    fn detect_default_target_arch_using_rustc(
        command_exec: &CommandExec,
    ) -> Result<CpuArchitecture> {
        command_exec
            .run("rustc", &["--print", "host-tuple"], None)
            .map_or_else(
                |e| Err(anyhow::anyhow!("Unable to read rustc host tuple: {e}")),
                |output| {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    match stdout.trim() {
                        "x86_64-pc-windows-msvc" => Ok(CpuArchitecture::Amd64),
                        "aarch64-pc-windows-msvc" => Ok(CpuArchitecture::Arm64),
                        _ => Err(anyhow::anyhow!(
                            "Unsupported default target: {}. Only x86_64-pc-windows-msvc and \
                             aarch64-pc-windows-msvc are supported.\n If you're on Windows, \
                             consider using the --target-arch option to specify a supported \
                             architecture.",
                            stdout
                        )),
                    }
                },
            )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::ref_option_ref)] // This is suppressed for mockall as it generates mocks with env_vars: &Option
    use std::{
        collections::HashMap,
        process::{ExitStatus, Output},
    };

    use mockall_double::double;
    use wdk_build::CpuArchitecture;

    use crate::cli::Cli;
    #[double]
    use crate::providers::exec::CommandExec;

    #[test]
    pub fn given_toolchain_host_tuple_is_x86_64_when_detect_default_arch_from_rustc_is_called_then_it_returns_arch(
    ) {
        let mut mock_command_exec = CommandExec::default();

        let expected_rustc_command = "rustc";
        let expected_rustc_args = vec!["--print", "host-tuple"];

        mock_command_exec
            .expect_run()
            .withf(
                move |command: &str,
                      args: &[&str],
                      _env_vars: &Option<&HashMap<&str, &str>>|
                      -> bool {
                    println!("command: {command}, args: {args:?}");
                    println!(
                        "expected_command: {expected_rustc_command}, expected_args: \
                         {expected_rustc_args:?}"
                    );
                    command == expected_rustc_command && args == expected_rustc_args
                },
            )
            .once()
            .returning(move |_, _, _| {
                Ok(Output {
                    status: ExitStatus::default(),
                    stdout: b"x86_64-pc-windows-msvc".to_vec(),
                    stderr: vec![],
                })
            });

        let result = Cli::detect_default_target_arch_using_rustc(&mock_command_exec);

        assert_eq!(result.unwrap(), CpuArchitecture::Amd64);
    }

    #[test]
    pub fn given_toolchain_host_tuple_is_aarch64_when_detect_default_arch_from_rustc_is_called_then_it_returns_arch(
    ) {
        let mut mock_command_exec = CommandExec::default();

        let expected_rustc_command = "rustc";
        let expected_rustc_args = vec!["--print", "host-tuple"];

        mock_command_exec
            .expect_run()
            .withf(
                move |command: &str,
                      args: &[&str],
                      _env_vars: &Option<&HashMap<&str, &str>>|
                      -> bool {
                    println!("command: {command}, args: {args:?}");
                    println!(
                        "expected_command: {expected_rustc_command}, expected_args: \
                         {expected_rustc_args:?}"
                    );
                    command == expected_rustc_command && args == expected_rustc_args
                },
            )
            .once()
            .returning(move |_, _, _| {
                Ok(Output {
                    status: ExitStatus::default(),
                    stdout: b"aarch64-pc-windows-msvc".to_vec(),
                    stderr: vec![],
                })
            });

        let result = Cli::detect_default_target_arch_using_rustc(&mock_command_exec);

        assert_eq!(result.unwrap(), CpuArchitecture::Arm64);
    }

    #[test]
    pub fn given_toolchain_host_tuple_is_i686_pc_windows_msvc_when_detect_default_arch_from_rustc_is_called_then_it_returns_error(
    ) {
        let mut mock_command_exec = CommandExec::default();

        let expected_rustc_command = "rustc";
        let expected_rustc_args = vec!["--print", "host-tuple"];

        mock_command_exec
            .expect_run()
            .withf(
                move |command: &str,
                      args: &[&str],
                      _env_vars: &Option<&HashMap<&str, &str>>|
                      -> bool {
                    println!("command: {command}, args: {args:?}");
                    println!(
                        "expected_command: {expected_rustc_command}, expected_args: \
                         {expected_rustc_args:?}"
                    );
                    command == expected_rustc_command && args == expected_rustc_args
                },
            )
            .once()
            .returning(move |_, _, _| {
                Ok(Output {
                    status: ExitStatus::default(),
                    stdout: b"i686-pc-windows-msvc".to_vec(),
                    stderr: vec![],
                })
            });

        let result = Cli::detect_default_target_arch_using_rustc(&mock_command_exec);

        assert_eq!(
            result.err().unwrap().to_string(),
            format!(
                "Unsupported default target: {}. Only x86_64-pc-windows-msvc and \
                 aarch64-pc-windows-msvc are supported.\n If you're on Windows, consider using \
                 the --target-arch option to specify a supported architecture.",
                "i686-pc-windows-msvc"
            )
        );
    }

    #[test]
    pub fn given_toolchain_host_tuple_is_x86_64_win7_windows_msvc_when_detect_default_arch_from_rustc_is_called_then_it_returns_error(
    ) {
        let mut mock_command_exec = CommandExec::default();

        let expected_rustc_command = "rustc";
        let expected_rustc_args = vec!["--print", "host-tuple"];

        mock_command_exec
            .expect_run()
            .withf(
                move |command: &str,
                      args: &[&str],
                      _env_vars: &Option<&HashMap<&str, &str>>|
                      -> bool {
                    println!("command: {command}, args: {args:?}");
                    println!(
                        "expected_command: {expected_rustc_command}, expected_args: \
                         {expected_rustc_args:?}"
                    );
                    command == expected_rustc_command && args == expected_rustc_args
                },
            )
            .once()
            .returning(move |_, _, _| {
                Ok(Output {
                    status: ExitStatus::default(),
                    stdout: b"x86_64-win7-windows-msvc".to_vec(),
                    stderr: vec![],
                })
            });

        let result = Cli::detect_default_target_arch_using_rustc(&mock_command_exec);

        assert_eq!(
            result.err().unwrap().to_string(),
            format!(
                "Unsupported default target: {}. Only x86_64-pc-windows-msvc and \
                 aarch64-pc-windows-msvc are supported.\n If you're on Windows, consider using \
                 the --target-arch option to specify a supported architecture.",
                "x86_64-win7-windows-msvc"
            )
        );
    }

    #[test]
    pub fn given_rustc_command_fails_when_detect_default_arch_from_rustc_is_called_then_it_returns_error(
    ) {
        let mut mock_command_exec = CommandExec::default();

        let expected_rustc_command = "rustc";
        let expected_rustc_args = vec!["--print", "host-tuple"];

        mock_command_exec
            .expect_run()
            .withf(
                move |command: &str,
                      args: &[&str],
                      _env_vars: &Option<&HashMap<&str, &str>>|
                      -> bool {
                    println!("command: {command}, args: {args:?}");
                    println!(
                        "expected_command: {expected_rustc_command}, expected_args: \
                         {expected_rustc_args:?}"
                    );
                    command == expected_rustc_command && args == expected_rustc_args
                },
            )
            .once()
            .returning(move |_, _, _| {
                Err(crate::providers::error::CommandError::CommandFailed {
                    command: "rustc".to_string(),
                    args: vec!["--print".to_string(), "host-tuple".to_string()],
                    stdout: "command error".to_string(),
                })
            });

        let result = Cli::detect_default_target_arch_using_rustc(&mock_command_exec);

        assert_eq!(
            result.err().unwrap().to_string(),
            format!(
                "Unable to read rustc host tuple: Command 'rustc' with args [\"--print\", \
                 \"host-tuple\"] failed \n STDOUT: {}",
                "command error"
            )
        );
    }
}
