use git2::Oid;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// What the last `version` run changed, recorded so `undo` can reverse it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastRunState {
    pub previous_version: String,
    pub new_version: String,
    pub commit_oid: Option<String>,
    pub catch_up_tag: Option<String>,
}

impl LastRunState {
    pub fn new(
        previous_version: &Version,
        new_version: &Version,
        commit_oid: Option<Oid>,
        catch_up_tag: Option<String>,
    ) -> Self {
        Self {
            previous_version: previous_version.to_string(),
            new_version: new_version.to_string(),
            commit_oid: commit_oid.map(|oid| oid.to_string()),
            catch_up_tag,
        }
    }
}

fn state_file_path(git_dir: &Path) -> PathBuf {
    git_dir.join("cargo-semantic-release-last-run.toml")
}

/// Persist `state` for a later `undo` to pick up.
pub fn write(git_dir: &Path, state: &LastRunState) -> Result<(), String> {
    let contents = toml::to_string(state).map_err(|error| error.to_string())?;
    std::fs::write(state_file_path(git_dir), contents).map_err(|error| error.to_string())
}

/// Read back the state left by the last `version` run.
///
/// ## Returns
///
/// `Ok(None)` if no `version` run has left a state file to undo.
pub fn read(git_dir: &Path) -> Result<Option<LastRunState>, String> {
    let path = state_file_path(git_dir);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    toml::from_str(&contents).map(Some).map_err(|error| error.to_string())
}

/// Remove the state file, e.g. after a successful `undo`.
pub fn delete(git_dir: &Path) -> Result<(), String> {
    match std::fs::remove_file(state_file_path(git_dir)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
mod last_run_state_tests {
    use super::LastRunState;
    use git2::Oid;
    use semver::Version;

    #[test]
    fn stringifies_the_versions_commit_oid_and_catch_up_tag() {
        // Given
        let previous_version = Version::new(1, 2, 3);
        let new_version = Version::new(1, 3, 0);
        let commit_oid = Oid::from_str("0123456789abcdef0123456789abcdef01234567").unwrap();

        // When
        let result = LastRunState::new(
            &previous_version,
            &new_version,
            Some(commit_oid),
            Some("v1.3.0".to_string()),
        );

        // Then
        assert_eq!(result.previous_version, "1.2.3");
        assert_eq!(result.new_version, "1.3.0");
        assert_eq!(
            result.commit_oid,
            Some("0123456789abcdef0123456789abcdef01234567".to_string())
        );
        assert_eq!(result.catch_up_tag, Some("v1.3.0".to_string()));
    }

    #[test]
    fn leaves_commit_oid_none_when_no_commit_was_made() {
        // Given / When
        let result = LastRunState::new(&Version::new(1, 0, 0), &Version::new(1, 1, 0), None, None);

        // Then
        assert_eq!(result.commit_oid, None);
        assert_eq!(result.catch_up_tag, None);
    }
}

#[cfg(test)]
mod tests {
    use super::{delete, read, write, LastRunState};
    use tempfile::TempDir;

    fn sample_state() -> LastRunState {
        LastRunState {
            previous_version: "1.2.3".to_string(),
            new_version: "1.3.0".to_string(),
            commit_oid: Some("abc123".to_string()),
            catch_up_tag: None,
        }
    }

    #[test]
    fn read_returns_none_when_no_state_file_exists() {
        // Given
        let temp_dir = TempDir::new().unwrap();

        // When
        let result = read(temp_dir.path());

        // Then
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn read_returns_what_was_written() {
        // Given
        let temp_dir = TempDir::new().unwrap();
        let state = sample_state();
        write(temp_dir.path(), &state).unwrap();

        // When
        let result = read(temp_dir.path());

        // Then
        assert_eq!(result.unwrap(), Some(state));
    }

    #[test]
    fn write_overwrites_a_previous_state_file() {
        // Given
        let temp_dir = TempDir::new().unwrap();
        write(temp_dir.path(), &sample_state()).unwrap();
        let updated_state = LastRunState {
            previous_version: "2.0.0".to_string(),
            new_version: "2.1.0".to_string(),
            commit_oid: None,
            catch_up_tag: Some("v2.0.0".to_string()),
        };

        // When
        write(temp_dir.path(), &updated_state).unwrap();

        // Then
        assert_eq!(read(temp_dir.path()).unwrap(), Some(updated_state));
    }

    #[test]
    fn delete_removes_the_state_file() {
        // Given
        let temp_dir = TempDir::new().unwrap();
        write(temp_dir.path(), &sample_state()).unwrap();

        // When
        delete(temp_dir.path()).unwrap();

        // Then
        assert_eq!(read(temp_dir.path()).unwrap(), None);
    }

    #[test]
    fn delete_is_a_no_op_when_the_state_file_is_already_gone() {
        // Given
        let temp_dir = TempDir::new().unwrap();

        // When
        let result = delete(temp_dir.path());

        // Then
        assert!(result.is_ok());
    }
}
