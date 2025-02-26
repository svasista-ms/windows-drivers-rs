use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use clap::Args;

use super::error::{InvalidDriverProjectNameError, NewProjectArgsError};
use crate::actions::{DriverType, Profile, TargetArch};

/// Type for Driver Project Name Argument
#[derive(Debug, Clone)]
pub struct ProjectNameArg(pub String);

impl FromStr for ProjectNameArg {
    type Err = NewProjectArgsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(NewProjectArgsError::InvalidDriverProjectNameError(
                s.to_string(),
                InvalidDriverProjectNameError::EmptyProjectNameError,
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(NewProjectArgsError::InvalidDriverProjectNameError(
                s.to_string(),
                InvalidDriverProjectNameError::NonAlphanumericProjectNameError,
            ));
        }
        if !s
            .chars()
            .next()
            .expect("Project name cannot be empty")
            .is_alphabetic()
        {
            return Err(NewProjectArgsError::InvalidDriverProjectNameError(
                s.to_string(),
                InvalidDriverProjectNameError::InvalidStartCharacter,
            ));
        }
        let invalid_names = ["crate", "self", "super", "extern", "_", "-", "new", "build"];
        if invalid_names.contains(&s) {
            return Err(NewProjectArgsError::InvalidDriverProjectNameError(
                s.to_string(),
                InvalidDriverProjectNameError::ReservedName(s.to_string()),
            ));
        }
        std::result::Result::Ok(Self(s.to_string()))
    }
}

/// Type for Driver Type Argument
#[derive(Debug, Clone)]
pub enum DriverTypeArg {
    Kmdf,
    Umdf,
    Wdm,
}

impl Into<DriverType> for DriverTypeArg {
    fn into(self) -> DriverType {
        match self {
            Self::Kmdf => DriverType::Kmdf,
            Self::Umdf => DriverType::Umdf,
            Self::Wdm => DriverType::Wdm,
        }
    }
}

impl FromStr for DriverTypeArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kmdf" => std::result::Result::Ok(Self::Kmdf),
            "umdf" => std::result::Result::Ok(Self::Umdf),
            "wdm" => std::result::Result::Ok(Self::Wdm),
            _ => Err(NewProjectArgsError::InvalidDriverTypeError(s.to_string()).to_string()),
        }
    }
}

/// Arguments for the new project subcommand
/// This struct is used to parse the command line arguments for creating a new
/// driver project.
#[derive(Debug, Args)]
pub struct NewProjectArgs {
    #[clap(help = "Driver Project Name")]
    pub driver_project_name: ProjectNameArg,
    #[clap(long, help = "Driver Type", index = 2, ignore_case = true)]
    pub driver_type: DriverTypeArg,
    #[clap(long, help = "Path to the project", default_value = ".")]
    pub cwd: PathBuf,
}

/// Type for Profile Argument
#[derive(Debug, Clone)]
pub enum ProfileArg {
    Debug,
    Release,
}

impl Into<Profile> for ProfileArg {
    fn into(self) -> Profile {
        match self {
            Self::Debug => Profile::Debug,
            Self::Release => Profile::Release,
        }
    }
}

impl FromStr for ProfileArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => std::result::Result::Ok(Self::Debug),
            "release" => std::result::Result::Ok(Self::Release),
            _ => Err(format!("'{s}' is not a valid profile")),
        }
    }
}

/// Type for Target Architecture Argument
#[derive(Debug, Clone)]
pub enum TargetArchArg {
    X64,
    Arm64,
}

impl Into<TargetArch> for TargetArchArg {
    fn into(self) -> TargetArch {
        match self {
            Self::X64 => TargetArch::X64,
            Self::Arm64 => TargetArch::Arm64,
        }
    }
}

impl FromStr for TargetArchArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x64" => std::result::Result::Ok(Self::X64),
            "arm64" => std::result::Result::Ok(Self::Arm64),
            _ => Err(format!("'{s}' is not a valid target architecture")),
        }
    }
}

/// Arguments for the package project subcommand
/// This struct is used to parse the command line arguments for packaging a
/// driver project.
#[derive(Debug, Args)]
pub struct PackageProjectArgs {
    #[clap(long, help = "Path to the project", default_value = ".")]
    pub cwd: PathBuf,
    #[clap(
        long,
        help = "Build Profile/Configuration",
        default_value = "debug",
        ignore_case = true
    )]
    pub profile: ProfileArg,
    #[clap(long, help = "Build Target", default_value = "x64", ignore_case = true)]
    pub target_arch: TargetArchArg,
    #[clap(long, help = "Verify Signatures", default_value = "false")]
    pub verify_signature: bool,
    #[clap(long, help = "Sample Class", default_value = "true")]
    pub sample_class: bool,
}
