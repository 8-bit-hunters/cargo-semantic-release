mod commit;
mod commit_fetcher;
mod version_tag;

use crate::repo::commit::Commit;
use crate::repo::commit_fetcher::{fetch_all_commits, fetch_commits_until};
use crate::repo::prelude::RepositoryExtension;
use crate::repo::version_tag::{get_all_version_tags, get_latest_version_tag};
use git2::{Oid, Repository};
use std::error::Error;
use std::path::Path;
use version_tag::VersionTag;

pub mod prelude {
    pub use crate::repo::commit::Commit;
    pub use crate::repo::commit::CommitInterface;
    pub use crate::repo::commit::{Gitmoji, GitmojiCommit};
    pub use crate::repo::version_tag::{create_release_tag, render_tag, VersionTag};
    use git2::Oid;
    use std::error::Error;
    use std::path::Path;

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
        /// Commit `path`'s current working-directory content as a new commit on `HEAD`, with
        /// `HEAD`'s current commit as its sole parent.
        fn commit_file(&self, path: &Path, message: &str) -> Result<Oid, Box<dyn Error>>;
        /// Delete the tag named `name`.
        fn delete_tag(&self, name: &str) -> Result<(), Box<dyn Error>>;
        /// Move `HEAD` (and the branch it points at) back to `oid`, without touching the index
        /// or working tree.
        fn reset_soft_to(&self, oid: Oid) -> Result<(), Box<dyn Error>>;
        /// The `Oid` of `oid`'s sole parent commit.
        fn commit_parent_oid(&self, oid: Oid) -> Result<Oid, Box<dyn Error>>;
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

    fn commit_file(&self, path: &Path, message: &str) -> Result<Oid, Box<dyn Error>> {
        let mut index = self.index()?;
        index.add_path(path)?;
        index.write()?;
        let tree = self.find_tree(index.write_tree()?)?;

        let signature = self.signature()?;
        let parent = self.head()?.peel_to_commit()?;

        Ok(self.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?)
    }

    fn delete_tag(&self, name: &str) -> Result<(), Box<dyn Error>> {
        Ok(self.tag_delete(name)?)
    }

    fn reset_soft_to(&self, oid: Oid) -> Result<(), Box<dyn Error>> {
        let object = self.find_object(oid, None)?;
        self.reset(&object, git2::ResetType::Soft, None)?;
        Ok(())
    }

    fn commit_parent_oid(&self, oid: Oid) -> Result<Oid, Box<dyn Error>> {
        Ok(self.find_commit(oid)?.parent_id(0)?)
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

    #[test]
    fn commit_file_creates_a_commit_with_the_given_message_as_head_s_new_parent() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let previous_head_oid = repository.head_commit_oid().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "version = \"2.0.0\"\n").unwrap();

        // When
        let result = repository
            .commit_file(
                std::path::Path::new("Cargo.toml"),
                ":bookmark: Bump release version to v2.0.0",
            )
            .unwrap();

        // Then
        assert_eq!(repository.head_commit_oid().unwrap(), result);
        let commit = repository.find_commit_by_message(":bookmark: Bump release version to v2.0.0");
        assert!(commit.is_some());
        assert_eq!(commit.unwrap().parent_id(0).unwrap(), previous_head_oid);
    }

    #[test]
    fn delete_tag_makes_it_undiscoverable_via_get_latest_version_tag() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let head_oid = repository.head_commit_oid().unwrap();
        repository.create_tag("v1.2.3", head_oid).unwrap();

        // When
        repository.delete_tag("v1.2.3").unwrap();

        // Then
        let result = repository.get_latest_version_tag("v{version}").unwrap();
        assert!(result.is_none(), "Expected None, but got Some");
    }

    #[test]
    fn reset_soft_to_moves_head_without_touching_the_working_tree() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let first_commit_oid = repository.head_commit_oid().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "version = \"2.0.0\"\n").unwrap();
        repository
            .commit_file(std::path::Path::new("Cargo.toml"), "bump version")
            .unwrap();

        // When
        repository.reset_soft_to(first_commit_oid).unwrap();

        // Then
        assert_eq!(repository.head_commit_oid().unwrap(), first_commit_oid);
        let working_tree_content =
            std::fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
        assert_eq!(working_tree_content, "version = \"2.0.0\"\n");
    }

    #[test]
    fn commit_parent_oid_returns_the_sole_parent_s_oid() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let parent_oid = repository.head_commit_oid().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "version = \"2.0.0\"\n").unwrap();
        let child_oid = repository
            .commit_file(std::path::Path::new("Cargo.toml"), "bump version")
            .unwrap();

        // When
        let result = repository.commit_parent_oid(child_oid).unwrap();

        // Then
        assert_eq!(result, parent_oid);
    }

    #[test]
    fn commit_file_includes_the_file_s_current_working_directory_content() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        std::fs::write(temp_dir.path().join("Cargo.toml"), "version = \"2.0.0\"\n").unwrap();

        // When
        let result = repository
            .commit_file(std::path::Path::new("Cargo.toml"), "bump")
            .unwrap();

        // Then
        let commit = repository.find_commit(result).unwrap();
        let tree = commit.tree().unwrap();
        let entry = tree.get_path(std::path::Path::new("Cargo.toml")).unwrap();
        let blob = repository.find_blob(entry.id()).unwrap();
        assert_eq!(blob.content(), b"version = \"2.0.0\"\n");
    }
}
