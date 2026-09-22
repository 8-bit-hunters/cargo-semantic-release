//! Cargo workspace implementation.
//!
//! This module provides a concrete implementation of the [`Workspace`] trait for
//! Cargo-based workspaces. It handles parsing Cargo manifests, discovering all
//! packages in a workspace (including root package and members), and validating
//! scopes against known package names.
//!
//! # Design
//!
//! The [`CargoWorkspace`] struct is the main entry point. It:
//! - Loads and parses the Cargo manifest from a given path
//! - Extracts package names from both the root package and workspace members
//! - Provides scope validation via the [`Workspace`] trait
//!
//! # Error Handling
//!
//! Errors are wrapped in [`CargoWorkspaceError`] and include:
//! - Manifest loading failures (IO errors, parse errors)
//! - Workspace with no valid packages
//! - Invalid paths (e.g., root filesystem with no parent)
//!
//! # Usage
//!
//! ```rust,ignore
//! use cargo_semantic_release::workspace::cargo::CargoWorkspace;
//! use std::path::Path;
//!
//! // Load workspace from Cargo.toml
//! let workspace = CargoWorkspace::from_path(Path::new("Cargo.toml"))?;
//!
//! // Get all package names
//! let packages = workspace.package_names();
//!
//! // Validate a commit scope
//! assert!(workspace.is_known_scope("my-crate"));
//! ```

use crate::workspace::Workspace;
use cargo_toml::Manifest;
use std::collections::HashSet;
use std::io::{Error as IoError, ErrorKind};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, trace, warn};

/// A workspace implementation for Cargo projects.
///
/// This struct represents a Cargo workspace and provides methods for
/// discovering package names and validating scopes.
///
/// # Fields
///
/// - `packages`: A set of all package names in the workspace (root + members)
///
/// # Example
///
/// ```rust,ignore
/// use std::path::Path;
///
/// let workspace = CargoWorkspace::from_path(Path::new("Cargo.toml"))?;
/// let packages = workspace.package_names();
/// ```
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CargoWorkspace {
    packages: HashSet<String>,
}

impl CargoWorkspace {
    /// Creates a new [`CargoWorkspace`] by loading and parsing the Cargo manifest at the given path.
    ///
    /// This method performs the following steps:
    ///
    /// 1. Loads the root manifest from the given path
    /// 2. Determines the base directory for resolving member paths
    /// 3. Collects package names from the root package and workspace members
    /// 4. Validates that at least one package was found
    ///
    /// # Errors
    ///
    /// Returns [`CargoWorkspaceError::ManifestLoadError`] if:
    /// - The manifest file cannot be read (IO error)
    /// - The manifest contains invalid TOML
    /// - The manifest path has no parent directory (needed for workspace member resolution)
    ///
    /// Returns [`CargoWorkspaceError::NoPackagesFound`] if no valid packages are found
    /// in the manifest (empty workspace or all members excluded).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::path::Path;
    ///
    /// let workspace = CargoWorkspace::from_path(Path::new("Cargo.toml"))?;
    /// let packages = workspace.package_names();
    /// ```
    #[allow(dead_code)]
    #[tracing::instrument(name = "load_workspace")]
    pub fn from_path(path: &Path) -> Result<Self, CargoWorkspaceError> {
        let manifest = Self::load_manifest(path)?;
        debug!("Successfully loaded manifest");

        let base_dir = Self::load_base_dir(path)?;
        trace!(base_dir = %base_dir.display(), "Base directory resolved");

        let mut packages = HashSet::new();

        if let Some(pkg) = manifest.package {
            trace!(package = %pkg.name, "Adding root package");
            packages.insert(pkg.name);
        }

        if let Some(workspace) = manifest.workspace {
            debug!(
                member_count = workspace.members.len(),
                "Processing workspace members"
            );
            Self::add_workspace_members(&mut packages, base_dir, &workspace)?;
        }

        debug!(package_count = packages.len(), packages = ?packages, "Workspace loaded");

        if packages.is_empty() {
            warn!(path = %path.display(), "No packages found in workspace");
            return Err(CargoWorkspaceError::NoPackagesFound(
                path.display().to_string(),
            ));
        }

        Ok(CargoWorkspace { packages })
    }

    /// Loads and parses the Cargo manifest from the given path.
    ///
    /// # Errors
    ///
    /// Returns [`CargoWorkspaceError::ManifestLoadError`] if the manifest cannot be loaded or parsed.
    fn load_manifest(path: &Path) -> Result<Manifest, CargoWorkspaceError> {
        trace!(path = %path.display(), "Loading manifest");
        Manifest::from_path(path).map_err(|e| {
            warn!(path = %path.display(), error = %e, "Failed to load manifest");
            CargoWorkspaceError::ManifestLoadError(path.display().to_string(), e)
        })
    }

    /// Gets the base directory for resolving workspace member paths.
    ///
    /// This is the parent directory of the manifest path, used to locate
    /// member Cargo.toml files.
    ///
    /// # Errors
    ///
    /// Returns [`CargoWorkspaceError::ManifestLoadError`] if the path has no parent
    /// (e.g., root filesystem path).
    fn load_base_dir(path: &Path) -> Result<&Path, CargoWorkspaceError> {
        trace!(path = %path.display(), "Getting base directory");
        path.parent().ok_or_else(|| {
            warn!(path = %path.display(), "Path has no parent");
            CargoWorkspaceError::ManifestLoadError(
                path.display().to_string(),
                cargo_toml::Error::Io(IoError::new(
                    ErrorKind::InvalidInput,
                    "manifest path has no parent",
                )),
            )
        })
    }

    /// Loads package names from workspace members and adds them to the packages set.
    ///
    /// This method:
    /// 1. Clones the member list from the workspace
    /// 2. Filters out excluded members
    /// 3. Loads each member's manifest and extracts the package name
    /// 4. Inserts all valid package names into the packages set
    ///
    /// # Errors
    ///
    /// Returns [`CargoWorkspaceError::ManifestLoadError`] if any member manifest cannot be loaded.
    fn add_workspace_members(
        packages: &mut HashSet<String>,
        base_dir: &Path,
        workspace: &cargo_toml::Workspace,
    ) -> Result<(), CargoWorkspaceError> {
        trace!(member_count = workspace.members.len(), "Processing members");

        let mut member_names: Vec<String> = workspace.members.clone();

        for excluded in &workspace.exclude {
            trace!(excluded = %excluded, "Applying exclusion filter");
            member_names.retain(|name| name != excluded);
        }

        for member_path in member_names {
            trace!(member = %member_path, "Loading member");
            let member_manifest_path = base_dir.join(&member_path).join("Cargo.toml");
            let package_name = Self::load_member_package_name(&member_manifest_path)?;
            trace!(member = %member_path, package = %package_name, "Member resolved");
            packages.insert(package_name);
        }

        Ok(())
    }

    /// Loads the package name from a manifest at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`CargoWorkspaceError::ManifestLoadError`] if the manifest cannot be loaded.
    /// Returns [`CargoWorkspaceError::NoPackagesFound`] if the manifest has no package name.
    fn load_member_package_name(path: &Path) -> Result<String, CargoWorkspaceError> {
        trace!(path = %path.display(), "Loading member package name");
        Self::load_manifest(path)?
            .package
            .map(|pkg| pkg.name)
            .ok_or_else(|| {
                warn!(path = %path.display(), "No package name found in manifest");
                CargoWorkspaceError::NoPackagesFound(path.display().to_string())
            })
    }
}

impl Workspace for CargoWorkspace {
    type Error = CargoWorkspaceError;

    fn package_names(&self) -> Vec<String> {
        self.packages.clone().into_iter().collect()
    }

    fn is_known_scope(&self, scope: &str) -> bool {
        self.packages.contains(scope)
    }
}

/// Error types for Cargo workspace operations.
///
/// All errors include context about what went wrong and where.
///
/// # Variants
///
/// - `ManifestLoadError`: Failed to load or parse a Cargo manifest, includes the path and source error
/// - `NoPackagesFound`: Workspace contains no valid packages (empty manifest or all members excluded)
#[derive(Debug, Error, Clone)]
pub enum CargoWorkspaceError {
    /// Failed to load a Cargo manifest at the given path.
    ///
    /// The first string is the path, the second is the underlying error from `cargo_toml`.
    #[error("Failed to load manifest at {0}")]
    ManifestLoadError(String, #[source] cargo_toml::Error),

    /// No valid packages were found in the workspace.
    ///
    /// This can happen if:
    /// - The manifest is a workspace with an empty members list
    /// - All members were excluded
    /// - The manifest has no package section and no workspace section
    #[error("No packages found in workspace at {0}")]
    NoPackagesFound(String),
}

#[cfg(test)]
mod test_cargo_workspace {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    mod test_package_names {
        use super::*;

        #[test]
        fn when_single_package() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_cargo_toml("[package]\nname = \"my-crate\"\nversion = \"1.0.0\"\n");

            // When
            let packages = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            assert_eq!(
                packages,
                vec!["my-crate"],
                "should return the single package name"
            );
        }

        #[test]
        fn when_workspace_with_members() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member("a")
                .with_member("b")
                .with_cargo_toml("[workspace]\nmembers = [\"a\", \"b\"]\n");

            // When
            let packages = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            let mut packages = packages;
            let mut expected = vec!["a", "b"];
            packages.sort();
            expected.sort();
            assert_eq!(packages, expected, "should return all workspace members");
        }

        #[test]
        fn when_root_package_plus_members() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member("a")
                .with_cargo_toml("[package]\nname = \"root\"\nversion = \"1.0.0\"\n\n[workspace]\nmembers = [\"a\"]\n");

            // When
            let packages = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            let mut packages = packages;
            let mut expected = vec!["a", "root"];
            packages.sort();
            expected.sort();
            assert_eq!(
                packages, expected,
                "should return both root package and workspace members"
            );
        }

        #[test]
        fn when_duplicate_members() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member("a")
                .with_cargo_toml("[workspace]\nmembers = [\"a\", \"a\"]\n");

            // When
            let packages = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            assert_eq!(packages, vec!["a"], "should deduplicate member list");
        }

        #[test]
        fn when_member_with_path() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member_named("crates/a", "a")
                .with_cargo_toml("[workspace]\nmembers = [\"crates/a\"]\n");

            // When
            let packages = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            assert_eq!(
                packages,
                vec!["a"],
                "should resolve member path to package name"
            );
        }

        #[test]
        fn when_workspace_with_exclude() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member("a")
                .with_member("b")
                .with_cargo_toml("[workspace]\nmembers = [\"a\", \"b\"]\nexclude = [\"b\"]\n");

            // When
            let packages = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            assert_eq!(packages, vec!["a"], "should exclude specified packages");
        }

        #[test]
        fn when_nested_path() {
            // Given
            let (_cargo_project, nested_cargo_path) =
                TestCargoProject::new().with_nested_package("crates/my-package", "my-package");

            // When
            let packages = CargoWorkspace::from_path(&nested_cargo_path)
                .expect("Failed to parse Cargo.toml")
                .package_names();

            // Then
            assert_eq!(
                packages,
                vec!["my-package"],
                "should handle nested manifest paths"
            );
        }
    }

    mod test_errors {
        use super::*;

        #[test]
        fn when_manifest_not_found() {
            // Given
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let nonexistent_path = temp_dir.path().join("nonexistent").join("Cargo.toml");
            let path_str = nonexistent_path.to_string_lossy().to_string();

            // When
            let result = CargoWorkspace::from_path(&nonexistent_path);

            // Then
            assert!(
                result.is_err(),
                "should return error when manifest not found"
            );
            let err = result.unwrap_err();
            assert!(
                matches!(err, CargoWorkspaceError::ManifestLoadError(p, _) if p == path_str),
                "error should be ManifestLoadError with correct path"
            );
        }

        #[test]
        fn when_invalid_toml_syntax() {
            // Given
            let cargo_project = TestCargoProject::new().with_cargo_toml("invalid toml syntax [[[");
            let cargo_path = cargo_project.cargo_toml();
            let path_str = cargo_path.to_string_lossy().to_string();

            // When
            let result = CargoWorkspace::from_path(cargo_path);

            // Then
            assert!(result.is_err(), "should return error for invalid TOML");
            let err = result.unwrap_err();
            assert!(
                matches!(err, CargoWorkspaceError::ManifestLoadError(p, _) if p == path_str),
                "error should be ManifestLoadError with correct path"
            );
        }

        #[test]
        fn when_empty_workspace_members() {
            // Given
            let cargo_project =
                TestCargoProject::new().with_cargo_toml("[workspace]\nmembers = []");
            let cargo_path = cargo_project.cargo_toml();
            let path_str = cargo_path.to_string_lossy().to_string();

            // When
            let result = CargoWorkspace::from_path(cargo_path);

            // Then
            assert!(
                result.is_err(),
                "should return error when no packages found"
            );
            let err = result.unwrap_err();
            assert!(
                matches!(err, CargoWorkspaceError::NoPackagesFound(p) if p == path_str),
                "error should be NoPackagesFound with correct path"
            );
        }
    }

    mod test_is_known_scope {
        use super::*;

        #[test]
        fn when_scope_is_known_package() {
            // Given
            let known_package = "my-crate";
            let cargo_project = TestCargoProject::new()
                .with_member(known_package)
                .with_cargo_toml("[workspace]\nmembers = [\"my-crate\"]\n");

            let workspace = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml");

            // When
            let result = workspace.is_known_scope(known_package);

            // Then
            assert!(result, "should return true for known package");
        }

        #[test]
        fn when_scope_is_unknown_package() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member("my-crate")
                .with_cargo_toml("[workspace]\nmembers = [\"my-crate\"]\n");

            let workspace = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml");

            // When
            let result = workspace.is_known_scope("unknown-crate");

            // Then
            assert!(!result, "should return false for unknown scope");
        }

        #[test]
        fn when_scope_with_subdirectory() {
            // Given
            let cargo_project = TestCargoProject::new()
                .with_member_named("crates/a", "a")
                .with_cargo_toml("[workspace]\nmembers = [\"crates/a\"]\n");

            let workspace = CargoWorkspace::from_path(cargo_project.cargo_toml())
                .expect("Failed to parse Cargo.toml");

            // When
            let result = workspace.is_known_scope("a");

            // Then
            assert!(result, "should return true for package name");
        }
    }

    /// Builder for creating temporary Cargo project structures for testing.
    ///
    /// Provides a fluent API to construct various project layouts:
    /// - Single package manifests
    /// - Workspace manifests with members
    /// - Nested package structures
    /// - Custom member paths and names
    struct TestCargoProject {
        temp_dir: TempDir,
        cargo_toml: PathBuf,
    }

    impl TestCargoProject {
        /// Creates a new test project with an empty temporary directory.
        /// The default Cargo.toml path is `<temp_dir>/Cargo.toml`.
        fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let cargo_toml = temp_dir.path().join("Cargo.toml");
            Self {
                temp_dir,
                cargo_toml,
            }
        }

        /// Writes the main Cargo.toml content for the project.
        ///
        /// # Arguments
        /// * `content` - The complete TOML content for the manifest
        ///
        /// # Example
        /// ```rust,ignore
        /// TestCargoProject::new()
        ///     .with_cargo_toml("[package]\nname = \"my-crate\"\n")
        /// ```
        fn with_cargo_toml(self, content: &str) -> Self {
            fs::write(&self.cargo_toml, content).expect("Failed to write Cargo.toml");
            self
        }

        /// Creates a workspace member at the given path.
        ///
        /// The member's package name will match its path.
        /// Creates the directory structure and writes a minimal Cargo.toml.
        ///
        /// # Arguments
        /// * `member` - The path to the member directory (relative to temp directory)
        ///
        /// # Example
        /// ```rust,ignore
        /// TestCargoProject::new()
        ///     .with_member("my-crate")  // Creates my-crate/Cargo.toml with name = "my-crate"
        /// ```
        fn with_member(self, member: &str) -> Self {
            let member_dir = self.temp_dir.path().join(member);
            fs::create_dir_all(&member_dir).expect("Failed to create member directory");
            fs::write(
                member_dir.join("Cargo.toml"),
                format!("[package]\nname = \"{}\"\nversion = \"1.0.0\"\n", member),
            )
            .expect("Failed to write Cargo.toml");
            self
        }

        /// Creates a workspace member with a custom package name.
        ///
        /// Use this when the member's path differs from its package name.
        /// For example: path = "crates/my-crate", name = "my-crate".
        ///
        /// # Arguments
        /// * `path` - The directory path for the member (relative to temp directory)
        /// * `name` - The package name to use in the member's Cargo.toml
        ///
        /// # Example
        /// ```rust,ignore
        /// TestCargoProject::new()
        ///     .with_member_named("crates/my-crate", "my-crate")
        /// ```
        fn with_member_named(self, path: &str, name: &str) -> Self {
            let member_dir = self.temp_dir.path().join(path);
            fs::create_dir_all(&member_dir).expect("Failed to create member directory");
            fs::write(
                member_dir.join("Cargo.toml"),
                format!("[package]\nname = \"{}\"\nversion = \"1.0.0\"\n", name),
            )
            .expect("Failed to write Cargo.toml");
            self
        }

        /// Creates a standalone package at a nested path.
        ///
        /// Returns a tuple of (self, cargo_path) where cargo_path is the full path
        /// to the created Cargo.toml. Use this for testing non-workspace scenarios
        /// where the manifest is at a nested location.
        ///
        /// # Arguments
        /// * `path` - The directory path for the package (relative to temp directory)
        /// * `name` - The package name to use in the Cargo.toml
        ///
        /// # Example
        /// ```rust,ignore
        /// let (project, cargo_path) = TestCargoProject::new()
        ///     .with_nested_package("crates/my-package", "my-package");
        /// // cargo_path points to <temp_dir>/crates/my-package/Cargo.toml
        /// ```
        fn with_nested_package(self, path: &str, name: &str) -> (Self, PathBuf) {
            let nested_dir = self.temp_dir.path().join(path);
            fs::create_dir_all(&nested_dir).expect("Failed to create nested directory");
            let cargo_path = nested_dir.join("Cargo.toml");
            fs::write(
                &cargo_path,
                format!("[package]\nname = \"{}\"\nversion = \"1.0.0\"\n", name),
            )
            .expect("Failed to write Cargo.toml");
            (self, cargo_path)
        }

        /// Returns the path to the main Cargo.toml for this project.
        fn cargo_toml(&self) -> &Path {
            &self.cargo_toml
        }
    }
}
