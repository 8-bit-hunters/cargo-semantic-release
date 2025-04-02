mod commit;
mod commit_fetcher;
mod version_tag;

use crate::repo::commit::{Commit};
use crate::repo::commit_fetcher::{fetch_all_commits, fetch_commits_until};
use crate::repo::prelude::RepositoryExtension;
use crate::repo::version_tag::get_latest_version_tag;
use git2::{Oid, Repository};
use std::error::Error;
use version_tag::VersionTag;

pub mod prelude {
    pub use crate::repo::commit::Commit;
    pub use crate::repo::commit::CommitInterface;
    pub use crate::repo::commit::{GitmojiCommit, Gitmoji};
    pub use crate::repo::version_tag::VersionTag;
    use git2::Oid;
    use std::error::Error;

    pub trait RepositoryExtension {
        fn fetch_commits_until(&self, stop_oid: Oid) -> Result<Vec<Commit>, Box<dyn Error>>;
        fn fetch_all_commits(&self) -> Result<Vec<Commit>, Box<dyn Error>>;
        fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>>;
    }
}

impl RepositoryExtension for Repository {
    fn fetch_commits_until(&self, stop_oid: Oid) -> Result<Vec<Commit>, Box<dyn Error>> {
        fetch_commits_until(self, stop_oid)
    }

    fn fetch_all_commits(&self) -> Result<Vec<Commit>, Box<dyn Error>> {
        fetch_all_commits(self)
    }

    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>> {
        get_latest_version_tag(self)
    }
}
