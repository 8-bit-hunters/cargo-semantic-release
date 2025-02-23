use git2::{ObjectType, Oid, Reference, Repository, Tag};
use regex::Regex;
use semver::Version;
use std::error::Error;

/// Get the latest version tag.
/// ## Returns
/// [`VersionTag`] containing the latest version tag.
pub fn get_latest_version_tag(
    repository: &Repository,
) -> Result<Option<VersionTag>, Box<dyn Error>> {
    let mut version_tags: Vec<VersionTag> = Vec::new();

    let references = repository.references()?;
    for reference in references {
        let reference = reference?;
        if reference.is_tag() {
            if let Some(oid) = reference.target() {
                let object = repository.find_object(oid, None)?;

                if let Ok(tag_object) = object.peel(ObjectType::Tag) {
                    if let Some(tag) = tag_object.as_tag() {
                        if let Some(version_tag) = VersionTag::from_annotated_tag(tag) {
                            version_tags.push(version_tag);
                        }
                    }
                } else if let Some(version_tag) = VersionTag::from_lightweight_tag(reference) {
                    version_tags.push(version_tag);
                }
            }
        }
    }

    Ok(version_tags.iter().max().cloned())
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
    fn from_lightweight_tag(reference: Reference) -> Option<Self> {
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
