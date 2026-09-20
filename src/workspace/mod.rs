//! Workspace support for Cargo projects.
//!
//! This module provides functionality for working with Cargo workspaces,
//! including discovering package names and validating commit scopes against
//! known workspace packages.
//!
//! # Architecture
//!
//! The module is designed around the [`Workspace`] trait, which defines a common
//! interface for workspace operations. The [`cargo`] module provides the concrete
//! implementation for Cargo-based workspaces.
//!
//! # Usage
//!
//! ```rust,ignore
//! use cargo_semantic_release::workspace::{Workspace, cargo::CargoWorkspace};
//! use std::path::Path;
//!
//! let workspace = CargoWorkspace::from_path(Path::new("Cargo.toml"))?;
//! let package_names = workspace.package_names();
//! let is_valid = workspace.is_known_scope("my-crate");
//! ```

pub mod cargo;

/// Trait defining workspace operations.
///
/// Implementations of this trait provide a way to interact with different types
/// of workspaces (currently only Cargo workspaces are supported).
///
/// # Type Parameters
///
/// - `Error`: The error type returned by this workspace implementation.
#[allow(dead_code)]
pub trait Workspace {
    /// The error type for this workspace implementation.
    type Error;

    /// Returns a list of all package names in the workspace.
    ///
    /// For Cargo workspaces, this includes:
    /// - The root package (if the manifest has a `[package]` section)
    /// - All member packages (if the manifest has a `[workspace]` section)
    ///
    /// Packages are deduplicated and returned in arbitrary order.
    fn package_names(&self) -> Vec<String>;

    /// Checks if the given scope is a known package in the workspace.
    ///
    /// This is used to validate commit scopes against workspace packages.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope name to validate (e.g., "my-crate" from commit message `(my-crate)`)
    ///
    /// # Returns
    ///
    /// `true` if the scope matches a package name in the workspace, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let workspace: impl Workspace = ...;
    /// assert!(workspace.is_known_scope("my-crate"));
    /// assert!(!workspace.is_known_scope("unknown-crate"));
    /// ```
    fn is_known_scope(&self, scope: &str) -> bool;
}
