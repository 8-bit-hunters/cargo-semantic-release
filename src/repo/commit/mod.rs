pub use crate::repo::commit::gitmoji::{Gitmoji, GitmojiCommit};
use thiserror::Error;

mod gitmoji;

pub trait CommitInterface {
    type Error;

    fn message(&self) -> &str;
    fn hash(&self) -> &str;
    fn intention(&self) -> &Gitmoji;
}

#[derive(Debug, Error, PartialEq)]
pub enum CommitError {
    #[error("Commit does not have a message")]
    MissingMessage,
    #[error("Commit message does not contain a valid Gitmoji intention")]
    MissingIntention,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Commit {
    pub message: String,
    pub hash: String,
}

impl From<git2::Commit<'_>> for Commit {
    fn from(value: git2::Commit) -> Self {
        Self {
            message: value.message().unwrap().to_string(),
            hash: value.id().to_string(),
        }
    }
}

#[cfg(test)]
mod conventional_commit_tests {
    use super::Commit;
    use crate::test_util::{repo_init, RepositoryTestExtensions};

    #[test]
    fn create_from_git2_commit() {
        // Given
        let commit_messages = vec!["initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let git2_commit = repository.find_commit_by_message("initial commit").unwrap();

        // When
        let result = Commit::from(git2_commit.clone());

        // Then
        let expected_result = Commit {
            message: git2_commit.message().unwrap().to_string(),
            hash: git2_commit.id().to_string(),
        };
        assert_eq!(result, expected_result)
    }
}
