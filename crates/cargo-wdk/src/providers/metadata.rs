// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0
//! This module provides a wrapper around the `cargo-metadata` crate, offering
//! methods to retrieve metadata about Cargo projects. The module leverages the
//! `mockall` crate to enable mocking of its methods, facilitating easier unit
//! testing.

// Warns the get_cargo_metadata_at_path method is not used, however it is used.
// The intellisense confusion seems to come from automock
#![allow(dead_code)]
#![allow(clippy::unused_self)]

use std::path::Path;

use clap_cargo::Features;
use mockall::automock;

use super::{cargo_path, error::MetadataError};

#[derive(Default)]
pub struct Metadata {}

#[automock]
impl Metadata {
    /// Get the Cargo metadata at a given path.
    ///
    /// This function executes the `cargo metadata` command to retrieve the
    /// metadata for the Cargo project located at the specified path. The
    /// metadata includes information about the project's dependencies,
    /// targets, and other relevant details.
    ///
    /// # Arguments
    ///
    /// * `working_dir` - A reference to a `Path` that specifies the path to the
    ///   working directory.
    /// * `other_options` - Additional command-line options (e.g. `--locked`)
    ///   that are forwarded to the `cargo metadata` command.
    /// * `features` - Feature selection forwarded to the `cargo metadata`
    ///   command via [`clap_cargo::Features::forward_metadata`].
    ///
    /// # Returns
    ///
    /// Returns the project metadata on success.
    ///
    /// # Errors
    ///
    /// Returns [`MetadataError::CargoPath`] if `CARGO` is missing or empty.
    /// Returns [`MetadataError::CargoMetadata`] if the
    /// `cargo metadata` command fails or its output cannot be parsed.
    pub fn get_cargo_metadata_at_path(
        &self,
        working_dir: &Path,
        other_options: Vec<String>,
        features: &Features,
    ) -> Result<cargo_metadata::Metadata, MetadataError> {
        let cargo = cargo_path()?;
        let mut cmd = cargo_metadata::MetadataCommand::new();
        cmd.cargo_path(&cargo)
            .current_dir(working_dir)
            .other_options(other_options);
        features.forward_metadata(&mut cmd);
        cmd.exec().map_err(MetadataError::from)
    }
}
