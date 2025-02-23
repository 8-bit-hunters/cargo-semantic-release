use git2::{Object, ObjectType, Oid, Reference, Repository, Tag};
use regex::Regex;
use semver::Version;
use std::error::Error;

pub trait RepositoryVersionTagExtension {
    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>>;
}

impl RepositoryVersionTagExtension for Repository {
    /// Get the latest version tag.
    /// ## Returns
    /// [`VersionTag`] containing the latest version tag.
    fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>> {
        let references: Vec<Reference> = self
            .references()?
            .filter_map(|reference| reference.ok())
            .collect();

        let version_tags: Vec<VersionTag> = references
            .iter()
            .filter(|reference| reference.is_tag())
            .filter_map(|reference| {
                reference.target().and_then(|oid| {
                    self.find_object(oid, None)
                        .ok()
                        .map(|object| (reference, object))
                })
            })
            .filter_map(|(reference, object)| {
                Tag::from_object(object)
                    .and_then(|tag| VersionTag::from_annotated_tag(&tag))
                    .or_else(|| VersionTag::from_lightweight_tag(reference))
            })
            .collect();

        Ok(version_tags.iter().max().cloned())
    }
}

trait AnnotatedTag {
    fn from_object(object: Object) -> Option<Tag>;
}

impl AnnotatedTag for Tag<'_> {
    /// Create [`Tag`] from [`Object`] type.
    ///
    /// ## Returns
    ///
    /// [`Tag`] if the object is annotated tag, `None` otherwise.
    fn from_object(object: Object<'_>) -> Option<Tag<'_>> {
        object
            .peel(ObjectType::Tag)
            .ok()
            .and_then(|tag_object| tag_object.as_tag().cloned())
    }
}

/// A structure that represent a version tag.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VersionTag {
    /// Semantic version parsed from the tag name.
    pub version: Version,
    /// Object ID of the commit that the tag points to.
    pub commit_oid: Oid,
}

impl VersionTag {
    /// Creates a [`VersionTag`] from an annotated git tag.
    ///
    /// ## Returns
    ///
    /// `Option` which is `Some` if the version tag is valid, `None` otherwise.
    fn from_annotated_tag(tag: &Tag) -> Option<Self> {
        let tag_name = tag.name().unwrap();
        if !Self::is_valid_version_tag(tag_name) {
            return None;
        }
        let version_number = tag_name.trim_start_matches("v");
        Some(Self {
            version: Version::parse(version_number).unwrap(),
            commit_oid: tag.target_id(),
        })
    }

    /// Creates a [`VersionTag`] from a lightweight git tag.
    ///
    /// ## Returns
    ///
    /// `Option` which is `Some` if the version tag is valid, `None` otherwise.
    fn from_lightweight_tag(reference: &Reference) -> Option<Self> {
        let tag_name = reference.shorthand().unwrap();
        if !Self::is_valid_version_tag(tag_name) {
            return None;
        }
        let version_number = tag_name.trim_start_matches("v");
        Some(Self {
            version: Version::parse(version_number).unwrap(),
            commit_oid: reference.target().unwrap(),
        })
    }

    fn is_valid_version_tag(tag_name: &str) -> bool {
        let version_regex = Regex::new(r"^v\d+\.\d+\.\d+$").unwrap();
        version_regex.is_match(tag_name)
    }
}

#[cfg(test)]
mod get_latest_version_tag_tests {
    use crate::test_util::repo_init;
    pub use crate::test_util::RepositoryTestExtensions;
    pub use crate::version_tag::RepositoryVersionTagExtension;
    use semver::Version;

    #[test]
    fn repository_does_not_have_tags() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

        // When
        let result = repository.get_latest_version_tag().unwrap();

        // Then
        assert!(result.is_none(), "Expected None, but got Some")
    }

    #[test]
    fn repository_does_not_have_version_tags() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec![":tada: initial release"]));
        let commit = repository.find_commit_by_message(":tada: initial release");
        repository.add_tag(commit.unwrap(), "tag_1");

        // When
        let result = repository.get_latest_version_tag().unwrap();

        // Then
        assert!(result.is_none(), "Expected None, but got Some")
    }

    #[test]
    fn repository_has_one_annotated_version_tag() {
        // Given
        let commit_message = ":tada: initial release";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let commit = repository.find_commit_by_message(commit_message);
        repository.add_tag(commit.unwrap(), "v1.0.0");

        // When
        let result = repository.get_latest_version_tag().unwrap().unwrap();

        // Then
        assert_eq!(result.version, Version::parse("1.0.0").unwrap());
        assert_eq!(
            result.commit_oid,
            repository
                .find_commit_by_message(commit_message)
                .unwrap()
                .id(),
            "Object IDs don't match"
        );
    }

    #[test]
    fn repository_has_one_not_annotated_version_tag() {
        // Given
        let commit_message = ":tada: initial release";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let commit = repository.find_commit_by_message(commit_message).unwrap();
        repository
            .tag_lightweight("v1.0.0", commit.as_object(), false)
            .unwrap();

        // When
        let result = repository.get_latest_version_tag().unwrap().unwrap();

        // Then
        assert_eq!(result.version, Version::parse("1.0.0").unwrap());
        assert_eq!(
            result.commit_oid,
            repository
                .find_commit_by_message(commit_message)
                .unwrap()
                .id(),
            "Object IDs don't match"
        );
    }

    #[test]
    fn repository_have_multiple_version_tags() {
        // Given
        let commit_messages = vec![
            ":tada: initial release",
            ":sparkles: new feature",
            ":boom: everything is broken",
        ];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));
        let tags = vec!["v1.0.0", "v1.1.0", "v2.0.0"];
        commit_messages
            .iter()
            .map(|commit| repository.find_commit_by_message(commit).unwrap())
            .zip(tags)
            .for_each(|(commit_id, tag)| repository.add_tag(commit_id, &tag));

        // When
        let result = repository.get_latest_version_tag().unwrap().unwrap();

        // Then
        assert_eq!(result.version, Version::parse("2.0.0").unwrap());
        assert_eq!(
            result.commit_oid,
            repository
                .find_commit_by_message(commit_messages.last().unwrap())
                .unwrap()
                .id(),
            "Object IDs don't match"
        );
    }
}
