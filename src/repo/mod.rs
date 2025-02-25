mod commit_fetcher;
mod conventional_commit;
mod version_tag;

use crate::repo::commit_fetcher::fetch_commits_since_last_version;
use crate::repo::version_tag::get_latest_version_tag;
pub use conventional_commit::ConventionalCommit;
use git2::Repository;
use std::error::Error;
pub use version_tag::VersionTag;

pub trait RepositoryExtension {
    fn fetch_commits_since_last_version(&self) -> Result<Vec<ConventionalCommit>, Box<dyn Error>>;
    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>>;
}

impl RepositoryExtension for Repository {
    fn fetch_commits_since_last_version(&self) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
        fetch_commits_since_last_version(self)
    }

    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>> {
        get_latest_version_tag(self)
    }
}
