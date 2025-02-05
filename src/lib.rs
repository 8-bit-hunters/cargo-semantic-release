use git2::{Commit, Repository};
use std::error::Error;
pub fn get_commits(repository: &Repository) -> Result<Vec<Commit>, Box<dyn Error>> {
    let mut revwalk = repository.revwalk()?;
    revwalk.push_head()?;

    Ok(revwalk
        .filter_map(|object_id| object_id.ok())
        .filter_map(|valid_object_id| repository.find_commit(valid_object_id).ok())
        .collect())
}

#[cfg(test)]
mod library_test {
    use crate::get_commits;
    use git2::{Commit, Repository, RepositoryInitOptions};
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

    fn compare_commits(commits: &Vec<Commit>, committed_messages: &Vec<&str>) -> bool {
        let collected_commit_messages: HashSet<_> =
            commits.iter().filter_map(|c| c.message()).collect();
        let committed_messages: HashSet<_> = committed_messages.iter().copied().collect();
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
            compare_commits(&result, &expected_commit_messages),
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
            compare_commits(&result, &commit_messages),
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
