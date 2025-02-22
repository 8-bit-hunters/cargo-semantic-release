use git2::{Commit, Repository, RepositoryInitOptions, Revwalk, Signature};
use tempfile::TempDir;

#[doc(hidden)]
#[allow(dead_code)]
/// Create an empty git repository in a temporary directory.
/// # Returns
/// The handler for the temporary directory and for the git repository.
pub fn repo_init() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(temp_dir.path(), &opts).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "name").unwrap();
    config.set_str("user.email", "email").unwrap();
    (temp_dir, repo)
}

#[doc(hidden)]
#[allow(dead_code)]
/// Add commit to a given repository.
/// ## Returns
/// The modified repository.
pub fn add_commit(repository: Repository, commit_messages: String) -> Repository {
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

#[doc(hidden)]
#[allow(dead_code)]
/// Add tag to a given commit.
/// ## Returns
/// The modified repository.
pub fn add_tag(repository: &Repository, commit: Commit, tag_name: &str) {
    let signature = Signature::now("name", "email").unwrap();
    let _ = repository.tag(tag_name, &commit.into_object(), &signature, "", false);
}

#[doc(hidden)]
#[allow(dead_code)]
/// Find a commit by its message
/// ## Result
/// The commit if it's found, None if it's not found
pub fn find_commit_by_message<'repo>(
    repository: &'repo Repository,
    commit_message: &str,
) -> Option<Commit<'repo>> {
    let mut revwalk: Revwalk = repository.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk.set_sorting(git2::Sort::TIME).unwrap();

    revwalk
        .map(|oid| repository.find_commit(oid.unwrap()).unwrap())
        .find(|commit| commit.message().unwrap().contains(commit_message))
}
