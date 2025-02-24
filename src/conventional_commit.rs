use std::fmt::Display;

/// A structure to represent a git commit.
///
/// Can be created with [`from_git2_commit`] method
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct ConventionalCommit {
    pub message: String,
}

impl ConventionalCommit {
    /// Create [`Commit`] from [`git2::Commit`] object.
    ///
    /// [`Commit`]: ConventionalCommit
    /// ['git2::Commit`]: git2::Commit
    pub fn from_git2_commit(commit: git2::Commit) -> Self {
        Self {
            message: commit.message().unwrap().to_string(),
        }
    }

    /// Return a reference to the `message` attribute
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ConventionalCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
