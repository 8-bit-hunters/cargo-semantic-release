use git2::{Repository, RepositoryInitOptions};
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
