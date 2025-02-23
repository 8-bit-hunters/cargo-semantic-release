use crate::conventional_commit::ConventionalCommit;
pub use crate::version_tag::RepositoryVersionTagExtension;
use git2::Oid;
use git2::Repository;
use std::error::Error;

/// Get the commit messages since the last version tag from a given git repository.
///
/// If the repository doesn't have version tags, then it will return all the commits.
///
/// ## Returns
/// A vector containing the commits or an error type if an error occurs.
pub fn fetch_commits_since_last_version(
    repository: &Repository,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    match repository.get_latest_version_tag()? {
        Some(version_tag) => fetch_commits_until(repository, version_tag.commit_oid),
        None => fetch_all_commits(repository),
    }
}

fn fetch_commits_until(
    repository: &Repository,
    stop_oid: Oid,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    general_fetch_commits_until(repository, Some(stop_oid))
}

fn fetch_all_commits(repository: &Repository) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
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
mod get_commits_functionality {
    use crate::commits::fetch_commits_since_last_version;
    use crate::conventional_commit::ConventionalCommit;
    use crate::test_util::repo_init;
    pub use crate::test_util::RepositoryTestExtensions;
    use std::collections::HashSet;

    #[doc(hidden)]
    /// Compare the result of `get_commits` function with the expected commit messages.
    /// ## Returns
    /// `true` if the result and expected commit messages are the same, `false` otherwise.
    pub fn compare(
        result_of_get_commits: &[ConventionalCommit],
        expected_commits: &[&str],
    ) -> bool {
        let collected_commit_messages: HashSet<_> =
            result_of_get_commits.iter().map(|c| c.message()).collect();
        let committed_messages: HashSet<_> = expected_commits.iter().copied().collect();
        collected_commit_messages == committed_messages
    }

    #[test]
    fn getting_commits_from_repo_with_one_commit_without_tags() {
        // Given
        let commit_messages = vec!["initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = fetch_commits_since_last_version(&repository).unwrap();

        // Then
        assert!(
            compare(&result, &commit_messages),
            "result = {:?}\nexpected messages = {:?}",
            result,
            commit_messages
        )
    }

    #[test]
    fn getting_commits_from_repo_with_multiple_commits_without_tags() {
        // Given
        let commit_messages = vec!["commit 1", "commit 2", "commit 3"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = fetch_commits_since_last_version(&repository).unwrap();

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
        let result = fetch_commits_since_last_version(&repository);

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
        repository.add_tag(
            repository
                .find_commit_by_message(commit_messages[0])
                .unwrap(),
            "v1.0.0",
        );
        repository.add_tag(
            repository
                .find_commit_by_message(commit_messages[1])
                .unwrap(),
            "v1.1.0",
        );
        repository.add_tag(
            repository
                .find_commit_by_message(commit_messages[2])
                .unwrap(),
            "v2.0.0",
        );

        // Then
        let result = fetch_commits_since_last_version(&repository).unwrap();

        let expected_commits = &commit_messages[3..];
        assert!(
            compare(&result, expected_commits),
            "result = {:?}\nexpected messages = {:?}",
            result,
            expected_commits
        )
    }

    #[test]
    fn getting_commits_with_lightweight_tag() {
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
        let _ = repository
            .tag_lightweight(
                "v1.0.0",
                repository
                    .find_commit_by_message(commit_messages[2])
                    .unwrap()
                    .as_object(),
                false,
            )
            .unwrap();

        // Then
        let result = fetch_commits_since_last_version(&repository).unwrap();

        let expected_commits = &commit_messages[3..];
        assert!(
            compare(&result, expected_commits),
            "result = {:?}\nexpected messages = {:?}",
            result,
            expected_commits
        )
    }
}
