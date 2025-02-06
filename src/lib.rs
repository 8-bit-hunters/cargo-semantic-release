use git2::{Commit, Repository};
use std::error::Error;

pub fn get_commits(repository: &Repository) -> Result<Vec<conventional::Commit>, Box<dyn Error>> {
    let mut revwalk = repository.revwalk()?;
    revwalk.push_head()?;

    let commits_in_repo: Vec<Commit> = revwalk
        .filter_map(|object_id| object_id.ok())
        .filter_map(|valid_object_id| repository.find_commit(valid_object_id).ok())
        .collect();

    Ok(commits_in_repo
        .into_iter()
        .map(|commit| conventional::Commit::from_git2_commit(commit))
        .collect())
}

pub mod conventional {

    #[derive(Clone, Debug)]
    pub struct Commit {
        message: String,
    }

    impl Commit {
        pub fn from_git2_commit(commit: git2::Commit) -> Self {
            Self {
                message: commit.message().unwrap().to_string(),
            }
        }

        pub fn message(&self) -> &str {
            &self.message
        }
    }
}
#[cfg(test)]
mod library_test {
    use crate::{conventional, get_commits};
    use git2::{Repository, RepositoryInitOptions};
    use std::collections::HashSet;
    use tempfile::TempDir;

    fn repo_init() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let mut opts = RepositoryInitOptions::new();
        opts.initial_head("main");
        let repo = Repository::init_opts(temp_dir.path(), &opts).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();
        (temp_dir, repo)
    }

    fn add_commit(repository: Repository, commit_messages: String) -> Repository {
        {
            let id = repository.index().unwrap().write_tree().unwrap();
            let tree = repository.find_tree(id).unwrap();
            let sig = repository.signature().unwrap();

            let parents = repository
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok());
            let parents = match &parents {
                Some(commit) => vec![commit],
                None => vec![],
            };

            let _ = repository.commit(
                Some("HEAD"),
                &sig,
                &sig,
                commit_messages.as_str(),
                &tree,
                &parents,
            );
        }

        repository
    }

    fn compare(
        result_of_get_commits: &Vec<conventional::Commit>,
        expected_commits: &Vec<&str>,
    ) -> bool {
        let collected_commit_messages: HashSet<_> =
            result_of_get_commits.iter().map(|c| c.message()).collect();
        let committed_messages: HashSet<_> = expected_commits.iter().copied().collect();
        collected_commit_messages == committed_messages
    }

    #[test]
    fn test_getting_commits_from_repo_with_one_commit() {
        // Given
        let (_temp_dir, repository) = repo_init();
        let repository = add_commit(repository, "initial_commit".to_string());
        // When
        let result = get_commits(&repository).unwrap();
        // Then
        let expected_commit_messages = vec!["initial_commit"];
        assert!(
            compare(&result, &expected_commit_messages),
            "result = {:?}\nexpected result = {:?}",
            result,
            expected_commit_messages
        )
    }

    #[test]
    fn test_getting_commits_from_repo_with_multiple_commits() {
        // Given
        let (_temp_dir, mut repository) = repo_init();
        let commit_messages = vec!["commit 1", "commit 2", "commit 3"];
        for commit_message in &commit_messages {
            repository = add_commit(repository, commit_message.to_string());
        }
        // When
        let result = get_commits(&repository).unwrap();
        // Then
        assert!(
            compare(&result, &commit_messages),
            "result = {:?}\ncommit_messages = {:?}",
            result,
            commit_messages
        )
    }

    #[test]
    fn test_getting_commits_from_empty_repo() {
        // Given
        let (_temp_dir, repository) = repo_init();
        // When
        let result = get_commits(&repository);
        // Then
        assert!(result.is_err(), "Expected and error, but got Ok")
    }
}
