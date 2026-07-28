use git2::{Object, ObjectType, Oid, Reference, Repository, Tag};
use regex::Regex;
use semver::Version;
use std::error::Error;

/// Get every tag matching `tag_format`.
///
/// `tag_format` describes the shape of version tags, e.g. `"v{version}"`; the
/// literal `{version}` placeholder marks where the semantic version sits.
/// ## Returns
/// All matching [`VersionTag`]s, in no particular order.
pub fn get_all_version_tags(
    repository: &Repository,
    tag_format: &str,
) -> Result<Vec<VersionTag>, Box<dyn Error>> {
    let references: Vec<Reference> = repository
        .references()?
        .filter_map(|reference| reference.ok())
        .collect();

    Ok(references
        .iter()
        .filter(|reference| reference.is_tag())
        .filter_map(|reference| {
            reference.target().and_then(|oid| {
                repository
                    .find_object(oid, None)
                    .ok()
                    .map(|object| (reference, object))
            })
        })
        .filter_map(|(reference, object)| {
            Tag::from_object(object)
                .and_then(|tag| VersionTag::from_annotated_tag(&tag, tag_format))
                .or_else(|| VersionTag::from_lightweight_tag(reference, tag_format))
        })
        .collect())
}

/// Get the latest version tag.
///
/// `tag_format` describes the shape of version tags, e.g. `"v{version}"`; the
/// literal `{version}` placeholder marks where the semantic version sits.
/// ## Returns
/// [`VersionTag`] containing the latest version tag.
pub fn get_latest_version_tag(
    repository: &Repository,
    tag_format: &str,
) -> Result<Option<VersionTag>, Box<dyn Error>> {
    Ok(get_all_version_tags(repository, tag_format)?
        .into_iter()
        .max())
}

/// Render `version` into a tag name, following `tag_format`.
///
/// The inverse of [`VersionTag::parse_version_from_tag_name`]: the literal `{version}`
/// placeholder in `tag_format` is replaced with `version`, e.g. `render_tag("v{version}", ...)`
/// for version `1.2.3` yields `"v1.2.3"`.
pub fn render_tag(tag_format: &str, version: &Version) -> String {
    tag_format.replace("{version}", &version.to_string())
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
    /// `Option` which is `Some` if the version tag matches `tag_format`, `None` otherwise.
    fn from_annotated_tag(tag: &Tag, tag_format: &str) -> Option<Self> {
        let tag_name = tag.name().unwrap();
        let version = Self::parse_version_from_tag_name(tag_name, tag_format)?;
        Some(Self {
            version,
            commit_oid: tag.target_id(),
        })
    }

    /// Creates a [`VersionTag`] from a lightweight git tag.
    ///
    /// ## Returns
    ///
    /// `Option` which is `Some` if the version tag matches `tag_format`, `None` otherwise.
    fn from_lightweight_tag(reference: &Reference, tag_format: &str) -> Option<Self> {
        let tag_name = reference.shorthand().unwrap();
        let version = Self::parse_version_from_tag_name(tag_name, tag_format)?;
        Some(Self {
            version,
            commit_oid: reference.target().unwrap(),
        })
    }

    /// Parse a [`Version`] out of `tag_name`, if it matches `tag_format`.
    ///
    /// `tag_format` must contain the literal `{version}` placeholder, e.g. `"v{version}"`;
    /// everything before/after it is matched as a literal prefix/suffix.
    fn parse_version_from_tag_name(tag_name: &str, tag_format: &str) -> Option<Version> {
        let (prefix, suffix) = tag_format
            .split_once("{version}")
            .unwrap_or((tag_format, ""));
        let pattern = format!(
            "^{}(\\d+\\.\\d+\\.\\d+){}$",
            regex::escape(prefix),
            regex::escape(suffix)
        );
        let captures = Regex::new(&pattern).ok()?.captures(tag_name)?;
        Version::parse(&captures[1]).ok()
    }
}

#[cfg(test)]
mod get_all_version_tags_tests {
    use crate::repo::prelude::RepositoryExtension;
    use crate::test_util::repo_init;
    pub use crate::test_util::RepositoryTestExtensions;
    use semver::Version;
    use std::collections::HashSet;

    #[test]
    fn returns_an_empty_vec_without_tags() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec![":tada: initial release"]));

        // When
        let result = repository.get_all_version_tags("v{version}").unwrap();

        // Then
        assert!(result.is_empty());
    }

    #[test]
    fn returns_every_matching_version_tag() {
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
            .for_each(|(commit_id, tag)| repository.add_tag(commit_id, tag));

        // When
        let result = repository.get_all_version_tags("v{version}").unwrap();

        // Then
        let versions: HashSet<Version> = result.into_iter().map(|tag| tag.version).collect();
        assert_eq!(
            versions,
            HashSet::from([
                Version::new(1, 0, 0),
                Version::new(1, 1, 0),
                Version::new(2, 0, 0),
            ])
        );
    }

    #[test]
    fn ignores_tags_that_do_not_match_the_format() {
        // Given
        let commit_message = ":tada: initial release";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let commit = repository.find_commit_by_message(commit_message);
        repository.add_tag(commit.unwrap(), "not-a-version-tag");

        // When
        let result = repository.get_all_version_tags("v{version}").unwrap();

        // Then
        assert!(result.is_empty());
    }
}

#[cfg(test)]
mod render_tag_tests {
    use crate::repo::version_tag::render_tag;
    use semver::Version;

    #[test]
    fn renders_the_default_tag_format() {
        // Given
        let version = Version::new(1, 2, 3);

        // When
        let result = render_tag("v{version}", &version);

        // Then
        assert_eq!(result, "v1.2.3");
    }

    #[test]
    fn renders_a_custom_tag_format() {
        // Given
        let version = Version::new(1, 2, 3);

        // When
        let result = render_tag("release-{version}", &version);

        // Then
        assert_eq!(result, "release-1.2.3");
    }
}

#[cfg(test)]
mod version_tag_tests {
    use crate::repo::prelude::RepositoryExtension;
    use crate::test_util::repo_init;
    pub use crate::test_util::RepositoryTestExtensions;
    use semver::Version;

    #[test]
    fn repository_does_not_have_tags() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

        // When
        let result = repository.get_latest_version_tag("v{version}").unwrap();

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
        let result = repository.get_latest_version_tag("v{version}").unwrap();

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
        let result = repository
            .get_latest_version_tag("v{version}")
            .unwrap()
            .unwrap();

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
        let result = repository
            .get_latest_version_tag("v{version}")
            .unwrap()
            .unwrap();

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
            .for_each(|(commit_id, tag)| repository.add_tag(commit_id, tag));

        // When
        let result = repository
            .get_latest_version_tag("v{version}")
            .unwrap()
            .unwrap();

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

    #[test]
    fn repository_has_version_tag_with_custom_format() {
        // Given
        let commit_message = ":tada: initial release";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let commit = repository.find_commit_by_message(commit_message);
        repository.add_tag(commit.unwrap(), "release-1.0.0");

        // When
        let result = repository
            .get_latest_version_tag("release-{version}")
            .unwrap()
            .unwrap();

        // Then
        assert_eq!(result.version, Version::parse("1.0.0").unwrap());
    }

    #[test]
    fn ignores_tags_that_do_not_match_the_custom_format() {
        // Given
        let commit_message = ":tada: initial release";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let commit = repository.find_commit_by_message(commit_message);
        repository.add_tag(commit.unwrap(), "v1.0.0");

        // When
        let result = repository
            .get_latest_version_tag("release-{version}")
            .unwrap();

        // Then
        assert!(result.is_none(), "Expected None, but got Some");
    }
}
