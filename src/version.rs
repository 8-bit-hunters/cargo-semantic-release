use cargo_toml::Manifest;
use regex::Regex;
use semver::Version;
use std::fs;
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

/// Write `version` as the `[package].version` declared in a `Cargo.toml` file.
///
/// Replaces only the `version = "..."` line inside the `[package]` table with a targeted
/// text substitution, so comments, formatting, and every other table (including a
/// similarly-named `version` key in `[dependencies]` or elsewhere) are left untouched.
///
/// ## Returns
///
/// `Ok(())` on success, or an error message if the file couldn't be read/written or has no
/// `[package].version` line to replace.
pub fn set_cargo_version(cargo_toml_path: &Path, version: &Version) -> Result<(), String> {
    let contents = fs::read_to_string(cargo_toml_path).map_err(|error| error.to_string())?;
    let updated = replace_package_version(&contents, version)
        .ok_or_else(|| "Cargo.toml does not declare a [package] version".to_string())?;
    fs::write(cargo_toml_path, updated).map_err(|error| error.to_string())
}

/// Replace the `version = "..."` line inside `contents`'s `[package]` table with `version`.
///
/// ## Returns
///
/// The updated file contents, or `None` if there's no `[package]` table or no `version` line
/// inside it.
fn replace_package_version(contents: &str, version: &Version) -> Option<String> {
    let package_start = contents.find("[package]")?;
    let section_body_start = package_start + "[package]".len();
    let next_section_offset = contents[section_body_start..]
        .find("\n[")
        .map(|offset| section_body_start + offset)
        .unwrap_or(contents.len());

    let package_section = &contents[section_body_start..next_section_offset];
    let version_line = Regex::new(r#"(?m)^version\s*=\s*"[^"]*""#).unwrap();
    let new_package_section =
        version_line.replacen(package_section, 1, format!("version = \"{version}\""));

    if new_package_section == package_section {
        return None;
    }

    Some(format!(
        "{}{}{}",
        &contents[..section_body_start],
        new_package_section,
        &contents[next_section_offset..]
    ))
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

#[cfg(test)]
mod set_cargo_version_tests {
    use crate::version::set_cargo_version;
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
    fn updates_the_declared_package_version() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml(
            "[package]\nname = \"foo\"\nversion = \"1.2.3\"\nedition = \"2021\"\n",
        );

        // When
        set_cargo_version(&cargo_toml_path, &Version::new(2, 0, 0)).unwrap();

        // Then
        let result = fs::read_to_string(&cargo_toml_path).unwrap();
        assert_eq!(
            result,
            "[package]\nname = \"foo\"\nversion = \"2.0.0\"\nedition = \"2021\"\n"
        );
    }

    #[test]
    fn preserves_comments_and_formatting_outside_the_version_line() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml(
            "# a helpful comment\n[package]\nname = \"foo\" # inline comment\nversion = \"1.2.3\"\n",
        );

        // When
        set_cargo_version(&cargo_toml_path, &Version::new(1, 2, 4)).unwrap();

        // Then
        let result = fs::read_to_string(&cargo_toml_path).unwrap();
        assert_eq!(
            result,
            "# a helpful comment\n[package]\nname = \"foo\" # inline comment\nversion = \"1.2.4\"\n"
        );
    }

    #[test]
    fn does_not_touch_a_similarly_named_version_key_in_another_section() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml(
            "[package]\nname = \"foo\"\nversion = \"1.2.3\"\n\n\
             [dependencies]\nserde = { version = \"1.0\" }\n",
        );

        // When
        set_cargo_version(&cargo_toml_path, &Version::new(1, 2, 4)).unwrap();

        // Then
        let result = fs::read_to_string(&cargo_toml_path).unwrap();
        assert_eq!(
            result,
            "[package]\nname = \"foo\"\nversion = \"1.2.4\"\n\n\
             [dependencies]\nserde = { version = \"1.0\" }\n"
        );
    }

    #[test]
    fn errors_when_package_section_is_missing() {
        // Given
        let (_temp_dir, cargo_toml_path) = write_cargo_toml("[workspace]\nmembers = []\n");

        // When
        let result = set_cargo_version(&cargo_toml_path, &Version::new(1, 0, 0));

        // Then
        assert!(result.is_err(), "Expected Err, got Ok");
    }

    #[test]
    fn errors_when_cargo_toml_is_missing() {
        // Given
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // When
        let result = set_cargo_version(&cargo_toml_path, &Version::new(1, 0, 0));

        // Then
        assert!(result.is_err(), "Expected Err, got Ok");
    }
}
