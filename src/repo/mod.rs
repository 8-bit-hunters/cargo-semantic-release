mod commit;
mod commit_fetcher;
mod version_tag;

use crate::repo::commit::Commit;
use crate::repo::commit_fetcher::{fetch_all_commits, fetch_commits_until};
use crate::repo::prelude::RepositoryExtension;
use crate::repo::version_tag::{get_all_version_tags, get_latest_version_tag};
use git2::{Oid, Repository};
use std::error::Error;
use version_tag::VersionTag;

pub mod prelude {
    pub use crate::repo::commit::Commit;
    pub use crate::repo::commit::CommitInterface;
    pub use crate::repo::commit::{Gitmoji, GitmojiCommit};
    pub use crate::repo::version_tag::{render_tag, VersionTag};
    use git2::Oid;
    use std::error::Error;

    pub trait RepositoryExtension {
        fn fetch_commits_until(&self, stop_oid: Oid) -> Result<Vec<Commit>, Box<dyn Error>>;
        fn fetch_all_commits(&self) -> Result<Vec<Commit>, Box<dyn Error>>;
        fn get_latest_version_tag(
            &self,
            tag_format: &str,
        ) -> Result<Option<VersionTag>, Box<dyn Error>>;
        /// Every tag matching `tag_format`, in no particular order.
        fn get_all_version_tags(&self, tag_format: &str)
            -> Result<Vec<VersionTag>, Box<dyn Error>>;
        /// The commit `HEAD` currently points at.
        fn head_commit_oid(&self) -> Result<Oid, Box<dyn Error>>;
        /// Create an annotated tag named `name` pointing at `target_oid`.
        fn create_tag(&self, name: &str, target_oid: Oid) -> Result<Oid, Box<dyn Error>>;
    }
}

impl RepositoryExtension for Repository {
    fn fetch_commits_until(&self, stop_oid: Oid) -> Result<Vec<Commit>, Box<dyn Error>> {
        fetch_commits_until(self, stop_oid)
    }

    fn fetch_all_commits(&self) -> Result<Vec<Commit>, Box<dyn Error>> {
        fetch_all_commits(self)
    }

    fn get_latest_version_tag(
        &self,
        tag_format: &str,
    ) -> Result<Option<VersionTag>, Box<dyn Error>> {
        get_latest_version_tag(self, tag_format)
    }

    fn get_all_version_tags(&self, tag_format: &str) -> Result<Vec<VersionTag>, Box<dyn Error>> {
        get_all_version_tags(self, tag_format)
    }

    fn head_commit_oid(&self) -> Result<Oid, Box<dyn Error>> {
        Ok(self.head()?.peel_to_commit()?.id())
    }

    fn create_tag(&self, name: &str, target_oid: Oid) -> Result<Oid, Box<dyn Error>> {
        let object = self.find_object(target_oid, None)?;
        let signature = self.signature()?;
        Ok(self.tag(name, &object, &signature, "", false)?)
    }
}

#[cfg(test)]
mod repository_extension_write_tests {
    use crate::repo::prelude::*;
    use crate::test_util::{repo_init, RepositoryTestExtensions};
    use semver::Version;

    #[test]
    fn head_commit_oid_returns_the_tip_commit() {
        // Given
        let commit_message = "initial commit";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let expected = repository
            .find_commit_by_message(commit_message)
            .unwrap()
            .id();

        // When
        let result = repository.head_commit_oid().unwrap();

        // Then
        assert_eq!(result, expected);
    }

    #[test]
    fn create_tag_makes_the_tag_discoverable_via_get_latest_version_tag() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let head_oid = repository.head_commit_oid().unwrap();

        // When
        repository.create_tag("v1.2.3", head_oid).unwrap();

        // Then
        let result = repository
            .get_latest_version_tag("v{version}")
            .unwrap()
            .unwrap();
        assert_eq!(result.version, Version::new(1, 2, 3));
        assert_eq!(result.commit_oid, head_oid);
    }
}
