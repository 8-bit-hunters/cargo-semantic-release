use cargo_toml::Manifest;
use semver::Version;
use std::path::Path;

/// Read the `[package].version` declared in a `Cargo.toml` file.
///
/// ## Returns
///
/// The parsed [`Version`], or an error message if the file couldn't be read/parsed,
/// has no `[package]` section, or the version is inherited from a workspace.
pub fn get_cargo_version(cargo_toml_path: &Path) -> Result<Version, String> {
    let manifest = Manifest::from_path(cargo_toml_path).map_err(|error| error.to_string())?;
    manifest
        .package
        .as_ref()
        .and_then(|package| package.version.get().ok())
        .cloned()
        .ok_or_else(|| "Cargo.toml does not declare a package version".to_string())
}

#[cfg(test)]
mod get_cargo_version_tests {
    use crate::version::get_cargo_version;
    use semver::Version;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_cargo_toml(contents: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml_path, contents).unwrap();
        (temp_dir, cargo_toml_path)
    }

    #[test]
    fn returns_the_declared_package_version() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml(
            "[package]\nname = \"foo\"\nversion = \"1.2.3\"\nedition = \"2021\"\n",
        );

        // When
        let result = get_cargo_version(&cargo_toml_path);

        // Then
        assert_eq!(result, Ok(Version::new(1, 2, 3)));
    }

    #[test]
    fn errors_when_cargo_toml_is_missing() {
        // Given
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // When
        let result = get_cargo_version(&cargo_toml_path);

        // Then
        assert!(result.is_err(), "Expected Err, got Ok");
    }

    #[test]
    fn errors_when_package_section_is_missing() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml("[workspace]\nmembers = []\n");

        // When
        let result = get_cargo_version(&cargo_toml_path);

        // Then
        assert!(result.is_err(), "Expected Err, got Ok");
    }

    #[test]
    fn errors_when_version_is_workspace_inherited() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml(
            "[package]\nname = \"foo\"\nversion.workspace = true\nedition = \"2021\"\n",
        );

        // When
        let result = get_cargo_version(&cargo_toml_path);

        // Then
        assert!(result.is_err(), "Expected Err, got Ok");
    }
}