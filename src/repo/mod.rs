mod commit_fetcher;
mod conventional_commit;
mod version_tag;

use crate::repo::commit_fetcher::{fetch_all_commits, fetch_commits_until};
use crate::repo::version_tag::get_latest_version_tag;
pub use conventional_commit::ConventionalCommit;
use git2::{Oid, Repository};
use std::error::Error;
pub use version_tag::VersionTag;

pub trait RepositoryExtension {
    fn fetch_commits_until(&self, stop_oid: Oid)
        -> Result<Vec<ConventionalCommit>, Box<dyn Error>>;
    fn fetch_all_commits(&self) -> Result<Vec<ConventionalCommit>, Box<dyn Error>>;
    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>>;
}

impl RepositoryExtension for Repository {
    fn fetch_commits_until(
        &self,
        stop_oid: Oid,
    ) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
        fetch_commits_until(self, stop_oid)
    }

    fn fetch_all_commits(&self) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
        fetch_all_commits(self)
    }

    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>> {
        get_latest_version_tag(self)
    }
}
