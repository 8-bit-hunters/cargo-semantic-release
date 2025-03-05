use std::fmt::Display;

/// A structure to represent a git commit.
///
/// Can be created with [`from_git2_commit`] method
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct ConventionalCommit {
    pub message: String,
    pub hash: String,
}

impl ConventionalCommit {
    /// Create [`Commit`] from [`git2::Commit`] object.
    ///
    /// [`Commit`]: ConventionalCommit
    /// ['git2::Commit`]: git2::Commit
    pub fn from_git2_commit(commit: git2::Commit) -> Self {
        Self {
            message: commit.message().unwrap().to_string(),
            hash: commit.id().to_string(),
        }
    }

    /// Return a reference to the `message` attribute
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ConventionalCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let short_hash = self
            .hash
            .get(0..7)
            .unwrap_or("Error: can't show short hash");
        write!(f, "{} - {}", self.message.trim_end(), short_hash)
    }
}

#[cfg(test)]
mod conventional_commit_tests {
    use crate::repo::ConventionalCommit;
    use crate::test_util::{repo_init, RepositoryTestExtensions};

    #[test]
    fn create_from_git2_commit() {
        // Given
        let commit_messages = vec!["initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let git2_commit = repository.find_commit_by_message("initial commit").unwrap();

        // When
        let result = ConventionalCommit::from_git2_commit(git2_commit.clone());

        // Then
        let expected_result = ConventionalCommit {
            message: git2_commit.message().unwrap().to_string(),
            hash: git2_commit.id().to_string(),
        };
        assert_eq!(result, expected_result)
    }

    #[test]
    fn display_formatting() {
        // Given
        let commit_messages = vec!["initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let git2_commit = repository.find_commit_by_message("initial commit").unwrap();
        let conventional_commit = ConventionalCommit::from_git2_commit(git2_commit.clone());

        // When
        let print_out = format!("{}", conventional_commit);

        // Then
        assert_eq!(
            print_out,
            format!(
                "{} - {}",
                git2_commit.message().unwrap(),
                git2_commit.id().to_string().get(0..7).unwrap()
            )
        )
    }
}
