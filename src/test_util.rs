use git2::{Commit, Repository, RepositoryInitOptions, Revwalk, Signature};
use std::error::Error;
use std::fmt;
use tempfile::TempDir;

#[doc(hidden)]
#[allow(dead_code)]
/// Create an empty git repository in a temporary directory.
/// # Returns
/// The handler for the temporary directory and for the git repository.
pub fn repo_init(commits: Option<Vec<&str>>) -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(temp_dir.path(), &opts).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "name").unwrap();
    config.set_str("user.email", "email").unwrap();

    if let Some(commits) = commits {
        commits.iter().for_each(|commit| repo.add_commit(commit))
    }

    (temp_dir, repo)
}

pub trait RepositoryTestExtensions {
    #[allow(dead_code)]
    fn add_commit(&self, commit_message: &str);
    #[allow(dead_code)]
    fn add_tag(&self, commit: Commit, tag_name: &str);
    #[allow(dead_code)]
    fn find_commit_by_message(&self, commit_message: &str) -> Option<Commit>;
}

impl RepositoryTestExtensions for Repository {
    #[doc(hidden)]
    #[allow(dead_code)]
    /// Add commit to a given repository.
    /// ## Returns
    /// The modified repository.
    fn add_commit(&self, commit_message: &str) {
        {
            let id = self.index().unwrap().write_tree().unwrap();
            let tree = self.find_tree(id).unwrap();
            let sig = self.signature().unwrap();

            let parents = self.head().ok().and_then(|head| head.peel_to_commit().ok());
            let parents = match &parents {
                Some(commit) => vec![commit],
                None => vec![],
            };

            let _ = self.commit(Some("HEAD"), &sig, &sig, commit_message, &tree, &parents);
        }
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Add tag to a given commit.
    /// ## Returns
    /// The modified repository.
    fn add_tag(&self, commit: Commit, tag_name: &str) {
        let signature = Signature::now("name", "email").unwrap();
        let _ = self.tag(tag_name, &commit.into_object(), &signature, "", false);
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    /// Find a commit by its message
    /// ## Result
    /// The commit if it's found, None if it's not found
    fn find_commit_by_message(&self, commit_message: &str) -> Option<Commit> {
        let mut revwalk: Revwalk = self.revwalk().unwrap();
        revwalk.push_head().unwrap();
        revwalk.set_sorting(git2::Sort::TIME).unwrap();

        revwalk
            .map(|oid| self.find_commit(oid.unwrap()).unwrap())
            .find(|commit| commit.message().unwrap().contains(commit_message))
    }
}

#[derive(Debug)]
pub struct MockError;

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mock error")
    }
}

impl Error for MockError {}
