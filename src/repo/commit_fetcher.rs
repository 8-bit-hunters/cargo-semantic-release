use crate::repo::ConventionalCommit;
use git2::Oid;
use git2::Repository;
use std::error::Error;

pub fn fetch_commits_until(
    repository: &Repository,
    stop_oid: Oid,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    general_fetch_commits_until(repository, Some(stop_oid))
}

pub fn fetch_all_commits(
    repository: &Repository,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    general_fetch_commits_until(repository, None)
}

fn general_fetch_commits_until(
    repository: &Repository,
    stop_oid: Option<Oid>,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    let mut revwalk = repository.revwalk()?;
    revwalk.push_head()?;

    Ok(revwalk
        .filter_map(|object_id| object_id.ok())
        .take_while(|oid| Some(*oid) != stop_oid)
        .filter_map(|oid| repository.find_commit(oid).ok())
        .map(|commit| ConventionalCommit::from_git2_commit(commit))
        .collect())
}

#[cfg(test)]
mod commit_fetcher_tests {
    use crate::repo::ConventionalCommit;
    pub use crate::repo::RepositoryExtension;
    use crate::test_util::repo_init;
    pub use crate::test_util::RepositoryTestExtensions;
    use std::collections::HashSet;

    #[doc(hidden)]
    /// Compare the result of `get_commits` function with the expected commit messages.
    /// ## Returns
    /// `true` if the result and expected commit messages are the same, `false` otherwise.
    fn compare(result_of_get_commits: &[ConventionalCommit], expected_commits: &[&str]) -> bool {
        let collected_commit_messages: HashSet<_> =
            result_of_get_commits.iter().map(|c| c.message()).collect();
        let committed_messages: HashSet<_> = expected_commits.iter().copied().collect();
        collected_commit_messages == committed_messages
    }

    #[test]
    fn getting_commits_from_repo_with_one_commit() {
        // Given
        let commit_messages = vec!["initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = repository.fetch_all_commits().unwrap();

        // Then
        assert!(
            compare(&result, &commit_messages),
            "result = {:?}\nexpected messages = {:?}",
            result,
            commit_messages
        )
    }

    #[test]
    fn getting_commits_from_repo_with_multiple_commits() {
        // Given
        let commit_messages = vec!["commit 1", "commit 2", "commit 3"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = repository.fetch_all_commits().unwrap();

        // Then
        assert!(
            compare(&result, &commit_messages),
            "result = {:?}\ncommit_messages = {:?}",
            result,
            commit_messages
        )
    }

    #[test]
    fn getting_commits_from_empty_repo() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

        // When
        let result = repository.fetch_all_commits();

        // Then
        assert!(result.is_err(), "Expected and error, but got Ok")
    }

    #[test]
    fn getting_commits_until_the_last_version_tag() {
        // Given
        let commit_messages = vec![
            ":tada: initial release",
            ":sparkles: new feature",
            ":boom: everything is broken",
            ":memo: add some documentation",
            ":recycle: refactor the code base",
            ":rocket: to the moon",
        ];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));
        let version_tagged_commit =
            repository.find_commit_by_message(":boom: everything is broken");

        // Then
        let result = repository
            .fetch_commits_until(version_tagged_commit.unwrap().id())
            .unwrap();

        let expected_commits = &commit_messages[3..];
        assert!(
            compare(&result, expected_commits),
            "result = {:?}\nexpected messages = {:?}",
            result,
            expected_commits
        )
    }
}
