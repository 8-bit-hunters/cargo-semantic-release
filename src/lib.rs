mod test_util;

use git2::{ObjectType, Oid, Reference, Repository, Tag};
use regex::Regex;
use semver::Version;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;

/// Structure that represents the changes in a git repository
#[derive(PartialEq, Debug, Hash)]
pub struct Changes {
    /// Vector of commits with major changes
    major: Vec<ConventionalCommit>,
    /// Vector of commits with minor changes
    minor: Vec<ConventionalCommit>,
    /// Vector of commits with patch changes
    patch: Vec<ConventionalCommit>,
    /// Vector of commits with other changes
    other: Vec<ConventionalCommit>,
}

impl Changes {
    /// Sort the commits from a given repo into `major`, `minor`, `patch` and `other`
    /// change categories according to their commit flags.
    ///
    /// Commits are fetched since the latest version tag. If there are no version tags yet
    /// then all the commits from the repository are fetched.
    ///
    /// ## Returns
    ///
    /// The [`Changes`] structure with the sorted commits or error type.
    ///
    /// ## Example
    /// ```
    /// use git2::Repository;
    /// use cargo_semantic_release::Changes;
    ///
    /// let git_repo = Repository::open(".").unwrap();
    ///
    /// let changes = Changes::from_repo(&git_repo);
    /// println!("changes: {changes}")
    /// ```
    pub fn from_repo(repository: &Repository) -> Self {
        let major_tags = [(":boom:", "💥")];
        let minor_tags = [
            (":sparkles:", "✨"),
            (":children_crossing:", "🚸"),
            (":lipstick:", "💄"),
            (":iphone:", "📱"),
            (":egg:", "🥚"),
            (":chart_with_upwards_trend:", "📈"),
            (":heavy_plus_sign:", "➕"),
            (":heavy_minus_sign:", "➖"),
            (":passport_control:", "🛂"),
        ];
        let patch_tags = [
            (":art:", "🎨"),
            (":ambulance:", "🚑️"),
            (":lock:", "🔒️"),
            (":bug:", "🐛"),
            (":zap:", "⚡️"),
            (":goal_net:", "🥅"),
            (":alien:", "👽️"),
            (":wheelchair:", "♿️"),
            (":speech_balloon:", "💬"),
            (":mag:", "🔍️"),
            (":fire:", "🔥"),
            (":white_check_mark:", "✅"),
            (":closed_lock_with_key:", "🔐"),
            (":rotating_light:", "🚨"),
            (":green_heart:", "💚"),
            (":arrow_down:", "⬇️"),
            (":arrow_up:", "⬆️"),
            (":pushpin:", "📌"),
            (":construction_worker:", "👷"),
            (":recycle:", "♻️"),
            (":wrench:", "🔧"),
            (":hammer:", "🔨"),
            (":globe_with_meridians:", "🌐"),
            (":package:", "📦️"),
            (":truck:", "🚚"),
            (":bento:", "🍱"),
            (":card_file_box:", "🗃️"),
            (":loud_sound:", "🔊"),
            (":mute:", "🔇"),
            (":building_construction:", "🏗️"),
            (":camera_flash:", "📸"),
            (":label:", "🏷️"),
            (":seedling:", "🌱"),
            (":triangular_flag_on_post:", "🚩"),
            (":dizzy:", "💫"),
            (":adhesive_bandage:", "🩹"),
            (":monocle_face:", "🧐"),
            (":necktie:", "👔"),
            (":stethoscope:", "🩺"),
            (":technologist:", "🧑‍💻"),
            (":thread:", "🧵"),
            (":safety_vest:", "🦺"),
        ];
        let other_tags = [
            (":memo:", "📝"),
            (":rocket:", "🚀"),
            (":tada:", "🎉"),
            (":bookmark:", "🔖"),
            (":construction:", "🚧"),
            (":pencil2:", "✏️"),
            (":poop:", "💩"),
            (":rewind:", "⏪️"),
            (":twisted_rightwards_arrows:", "🔀"),
            (":page_facing_up:", "📄"),
            (":bulb:", "💡"),
            (":beers:", "🍻"),
            (":bust_in_silhouette:", "👥"),
            (":clown_face:", "🤡"),
            (":see_no_evil:", "🙈"),
            (":alembic:", "⚗️"),
            (":wastebasket:", "🗑️"),
            (":coffin:", "⚰️"),
            (":test_tube:", "🧪"),
            (":bricks:", "🧱"),
            (":money_with_wings:", "💸"),
        ];

        match get_commits_since_last_version(repository) {
            Ok(unsorted_commits) => Self {
                major: get_commits_with_tag(unsorted_commits.clone(), major_tags.to_vec()),
                minor: get_commits_with_tag(unsorted_commits.clone(), minor_tags.to_vec()),
                patch: get_commits_with_tag(unsorted_commits.clone(), patch_tags.to_vec()),
                other: get_commits_with_tag(unsorted_commits, other_tags.to_vec()),
            },
            Err(_) => Self {
                major: Vec::new(),
                minor: Vec::new(),
                patch: Vec::new(),
                other: Vec::new(),
            },
        }
    }

    /// Evaluate the changes find in a repository to figure out the semantic version action
    ///
    /// ## Returns
    ///
    /// [`SemanticVersionAction`] enum for the suggested semantic version change.
    ///
    /// ## Example
    ///
    /// ```
    ///  use git2::Repository;
    ///  use cargo_semantic_release::Changes;
    ///
    ///  let git_repo = Repository::open(".").unwrap();
    ///
    ///  let action = Changes::from_repo(&git_repo).define_action_for_semantic_version();
    ///  println!("suggested change of semantic version: {}", action);
    /// ```
    pub fn define_action_for_semantic_version(self) -> SemanticVersionAction {
        if !self.major.is_empty() {
            return SemanticVersionAction::IncrementMajor;
        }
        if !self.minor.is_empty() {
            return SemanticVersionAction::IncrementMinor;
        }
        if !self.patch.is_empty() {
            return SemanticVersionAction::IncrementPatch;
        }
        SemanticVersionAction::Keep
    }

    /// Compare two [`Changes`] struct to see if they have the same elements.
    ///
    /// # Returns
    ///
    /// `true` if the two structure has the same elements regardless they order, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use git2::Repository;
    /// use cargo_semantic_release::Changes;
    ///
    /// let git_repo = Repository::open(".").unwrap();
    ///
    /// let changes_1 = Changes::from_repo(&git_repo);
    /// let changes_2 = Changes::from_repo(&git_repo);
    ///
    /// let result = changes_1.has_same_elements(&changes_2);
    /// println!("{result}")
    /// ```
    pub fn has_same_elements(&self, other: &Self) -> bool {
        self.major.iter().collect::<HashSet<_>>() == other.major.iter().collect::<HashSet<_>>()
            && self.minor.iter().collect::<HashSet<_>>()
                == other.minor.iter().collect::<HashSet<_>>()
            && self.patch.iter().collect::<HashSet<_>>()
                == other.patch.iter().collect::<HashSet<_>>()
            && self.other.iter().collect::<HashSet<_>>()
                == other.other.iter().collect::<HashSet<_>>()
    }
}

impl Display for Changes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let major_changes = convert_to_string_vector(self.major.clone());
        let minor_changes = convert_to_string_vector(self.minor.clone());
        let patch_changes = convert_to_string_vector(self.patch.clone());
        let other_changes = convert_to_string_vector(self.other.clone());
        write!(
            f,
            "major:\n\t{}\nminor:\n\t{}\npatch:\n\t{}\nother:\n\t{}",
            major_changes.join("\t"),
            minor_changes.join("\t"),
            patch_changes.join("\t"),
            other_changes.join("\t")
        )
    }
}

/// Enum to represent the action for semantic version
#[derive(PartialEq, Debug)]
pub enum SemanticVersionAction {
    IncrementMajor,
    IncrementMinor,
    IncrementPatch,
    Keep,
}

impl Display for SemanticVersionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            SemanticVersionAction::IncrementMajor => "increment major version",
            SemanticVersionAction::IncrementMinor => "increment minor version",
            SemanticVersionAction::IncrementPatch => "increment patch version",
            SemanticVersionAction::Keep => "keep version",
        };
        write!(f, "{}", msg)
    }
}

/// A structure to represent a git commit.
///
/// Can be created with [`from_git2_commit`] method
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
struct ConventionalCommit {
    message: String,
}

impl ConventionalCommit {
    /// Create [`Commit`] from [`git2::Commit`] object.
    ///
    /// [`Commit`]: ConventionalCommit
    /// ['git2::Commit`]: git2::Commit
    pub fn from_git2_commit(commit: git2::Commit) -> Self {
        Self {
            message: commit.message().unwrap().to_string(),
        }
    }

    /// Return a reference to the `message` attribute
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ConventionalCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn convert_to_string_vector(commits: Vec<ConventionalCommit>) -> Vec<String> {
    commits
        .into_iter()
        .map(|commit| commit.message().to_string())
        .collect::<Vec<String>>()
}

fn get_commits_with_tag(
    commits: Vec<ConventionalCommit>,
    tags: Vec<(&str, &str)>,
) -> Vec<ConventionalCommit> {
    commits
        .into_iter()
        .filter(|commit| {
            tags.iter()
                .any(|tag| commit.message.contains(tag.0) || commit.message.contains(tag.1))
        })
        .collect()
}

/// Get the commit messages since the last version tag from a given git repository.
///
/// If the repository doesn't have version tags, then it will return all the commits.
///
/// ## Returns
/// A vector containing the commits or an error type if an error occurs.
fn get_commits_since_last_version(
    repository: &Repository,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    match get_latest_version_tag(repository)? {
        Some(version_tag) => fetch_commits_until(repository, version_tag.commit_oid),
        None => fetch_all_commits(repository),
    }
}

fn fetch_commits_until(
    repository: &Repository,
    stop_iod: Oid,
) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    let mut revwalk = repository.revwalk()?;
    revwalk.push_head()?;

    Ok(revwalk
        .filter_map(|object_id| object_id.ok())
        .take_while(|oid| *oid != stop_iod)
        .filter_map(|oid| repository.find_commit(oid).ok())
        .map(|commit| ConventionalCommit::from_git2_commit(commit))
        .collect())
}

fn fetch_all_commits(repository: &Repository) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
    let mut revwalk = repository.revwalk()?;
    revwalk.push_head()?;

    Ok(revwalk
        .filter_map(|object_id| object_id.ok())
        .filter_map(|valid_oid| repository.find_commit(valid_oid).ok())
        .map(|commit| ConventionalCommit::from_git2_commit(commit))
        .collect())
}

/// Get the latest version tag.
/// ## Returns
/// [`VersionTag`] containing the latest version tag.
fn get_latest_version_tag(repository: &Repository) -> Result<Option<VersionTag>, Box<dyn Error>> {
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
struct VersionTag {
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

#[cfg(test)]
mod get_latest_version_tag_tests {
    use crate::get_latest_version_tag;
    use crate::test_util::{add_commit, add_tag, find_commit_by_message, repo_init};
    use semver::Version;

    #[test]
    fn repository_does_not_have_tags() {
        // Given
        let (_temp_dir, repository) = repo_init();

        // When
        let result = get_latest_version_tag(&repository).unwrap();

        // Then
        assert!(result.is_none(), "Expected None, but got Some")
    }

    #[test]
    fn repository_does_not_have_version_tags() {
        // Given
        let (_temp_dir, mut repository) = repo_init();
        repository = add_commit(repository, ":tada: initial release".to_string());
        let commit = find_commit_by_message(&repository, ":tada: initial release");
        add_tag(&repository, commit.unwrap(), "tag_1");

        // When
        let result = get_latest_version_tag(&repository).unwrap();

        // Then
        assert!(result.is_none(), "Expected None, but got Some")
    }

    #[test]
    fn repository_has_one_annotated_version_tag() {
        let (_temp_dir, mut repository) = repo_init();
        let commit_message = ":tada: initial release";
        repository = add_commit(repository, commit_message.to_string());
        let commit = find_commit_by_message(&repository, commit_message);
        add_tag(&repository, commit.unwrap(), "v1.0.0");

        // When
        let result = get_latest_version_tag(&repository).unwrap().unwrap();

        // Then
        assert_eq!(result.version, Version::parse("1.0.0").unwrap());
        assert_eq!(
            result.commit_oid,
            find_commit_by_message(&repository, commit_message)
                .unwrap()
                .id(),
            "Object IDs don't match"
        );
    }

    #[test]
    fn repository_has_one_not_annotated_version_tag() {
        let (_temp_dir, mut repository) = repo_init();
        let commit_message = ":tada: initial release";
        repository = add_commit(repository, commit_message.to_string());
        let commit = find_commit_by_message(&repository, commit_message).unwrap();
        repository
            .tag_lightweight("v1.0.0", commit.as_object(), false)
            .unwrap();

        // When
        let result = get_latest_version_tag(&repository).unwrap().unwrap();

        // Then
        assert_eq!(result.version, Version::parse("1.0.0").unwrap());
        assert_eq!(
            result.commit_oid,
            find_commit_by_message(&repository, commit_message)
                .unwrap()
                .id(),
            "Object IDs don't match"
        );
    }

    #[test]
    fn repository_have_multiple_version_tags() {
        // Given
        let (_temp_dir, mut repository) = repo_init();
        let commit_messages = vec![
            ":tada: initial release",
            ":sparkles: new feature",
            ":boom: everything is broken",
        ];
        let tags = vec!["v1.0.0", "v1.1.0", "v2.0.0"];
        for commit_message in &commit_messages {
            repository = add_commit(repository, commit_message.to_string());
        }
        commit_messages
            .iter()
            .map(|commit| find_commit_by_message(&repository, commit).unwrap())
            .zip(tags)
            .for_each(|(commit_id, tag)| add_tag(&repository, commit_id, &tag));

        // When
        let result = get_latest_version_tag(&repository).unwrap().unwrap();

        // Then
        assert_eq!(result.version, Version::parse("2.0.0").unwrap());
        assert_eq!(
            result.commit_oid,
            find_commit_by_message(&repository, commit_messages.last().unwrap())
                .unwrap()
                .id(),
            "Object IDs don't match"
        );
    }
}

#[cfg(test)]
mod get_commits_functionality {
    use crate::test_util::{add_commit, add_tag, find_commit_by_message, repo_init};
    use crate::{get_commits_since_last_version, ConventionalCommit};
    use std::collections::HashSet;

    #[doc(hidden)]
    /// Compare the result of `get_commits` function with the expected commit messages.
    /// ## Returns
    /// `true` if the result and expected commit messages are the same, `false` otherwise.
    pub fn compare(
        result_of_get_commits: &[ConventionalCommit],
        expected_commits: &[&str],
    ) -> bool {
        let collected_commit_messages: HashSet<_> =
            result_of_get_commits.iter().map(|c| c.message()).collect();
        let committed_messages: HashSet<_> = expected_commits.iter().copied().collect();
        collected_commit_messages == committed_messages
    }

    #[test]
    fn getting_commits_from_repo_with_one_commit_without_tags() {
        // Given
        let (_temp_dir, repository) = repo_init();
        let repository = add_commit(repository, "initial_commit".to_string());
        // When
        let result = get_commits_since_last_version(&repository).unwrap();
        // Then
        let expected_commit_messages = vec!["initial_commit"];
        assert!(
            compare(&result, &expected_commit_messages),
            "result = {:?}\nexpected result = {:?}",
            result,
            expected_commit_messages
        )
    }

    #[test]
    fn getting_commits_from_repo_with_multiple_commits_without_tags() {
        // Given
        let (_temp_dir, mut repository) = repo_init();
        let commit_messages = vec!["commit 1", "commit 2", "commit 3"];
        for commit_message in &commit_messages {
            repository = add_commit(repository, commit_message.to_string());
        }
        // When
        let result = get_commits_since_last_version(&repository).unwrap();
        // Then
        assert!(
            compare(&result, &commit_messages),
            "result = {:?}\ncommit_messages = {:?}",
            result,
            commit_messages
        )
    }

    #[test]
    fn getting_commits_from_empty_repo() {
        // Given
        let (_temp_dir, repository) = repo_init();
        // When
        let result = get_commits_since_last_version(&repository);
        // Then
        assert!(result.is_err(), "Expected and error, but got Ok")
    }

    #[test]
    fn getting_commits_until_the_last_version_tag() {
        // Given
        let (_temp_dir, mut repository) = repo_init();
        let commit_messages = vec![
            ":tada: initial release",
            ":sparkles: new feature",
            ":boom: everything is broken",
            ":memo: add some documentation",
            ":recycle: refactor the code base",
            ":rocket: to the moon",
        ];
        for commit_message in &commit_messages {
            let message = *commit_message;
            repository = add_commit(repository, message.to_string());
        }
        add_tag(
            &repository,
            find_commit_by_message(&repository, commit_messages[0]).unwrap(),
            "v1.0.0",
        );
        add_tag(
            &repository,
            find_commit_by_message(&repository, commit_messages[1]).unwrap(),
            "v1.1.0",
        );
        add_tag(
            &repository,
            find_commit_by_message(&repository, commit_messages[2]).unwrap(),
            "v2.0.0",
        );

        // Then
        let result = get_commits_since_last_version(&repository).unwrap();

        let expected_commits = &commit_messages[3..];
        assert!(
            compare(&result, expected_commits),
            "result = {:?}\nexpected messages = {:?}",
            result,
            expected_commits
        )
    }

    #[test]
    fn getting_commits_with_lightweight_tag() {
        // Given
        let (_temp_dir, mut repository) = repo_init();
        let commit_messages = vec![
            ":tada: initial release",
            ":sparkles: new feature",
            ":boom: everything is broken",
            ":memo: add some documentation",
            ":recycle: refactor the code base",
            ":rocket: to the moon",
        ];
        for commit_message in &commit_messages {
            let message = *commit_message;
            repository = add_commit(repository, message.to_string());
        }
        let _ = repository
            .tag_lightweight(
                "v1.0.0",
                find_commit_by_message(&repository, commit_messages[2])
                    .unwrap()
                    .as_object(),
                false,
            )
            .unwrap();

        // Then
        let result = get_commits_since_last_version(&repository).unwrap();

        let expected_commits = &commit_messages[3..];
        assert!(
            compare(&result, expected_commits),
            "result = {:?}\nexpected messages = {:?}",
            result,
            expected_commits
        )
    }
}

#[cfg(test)]
mod changes_struct {
    use crate::test_util::{add_commit, repo_init};
    use crate::Changes;
    use crate::ConventionalCommit;

    #[test]
    fn creating_from_empty_commit_list() {
        // Given
        let (_temp_dir, repository) = repo_init();

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_major_conventional_commits() {
        // Given
        let commits = vec![ConventionalCommit {
            message: "💥 introduce breaking changes".to_string(),
        }];
        let (_temp_dir, mut repository) = repo_init();
        for commit in &commits {
            repository = add_commit(repository, commit.message.to_string());
        }

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: commits,
            minor: Vec::new(),
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_minor_conventional_commits() {
        // Given
        let commits = vec![
            ConventionalCommit {
                message: ":sparkles: introduce new feature".to_string(),
            },
            ConventionalCommit {
                message: ":children_crossing: improve user experience / usability".to_string(),
            },
            ConventionalCommit {
                message: "💄 add or update the UI and style files".to_string(),
            },
            ConventionalCommit {
                message: ":iphone: work on responsive design".to_string(),
            },
            ConventionalCommit {
                message: ":egg: add or update an easter egg".to_string(),
            },
            ConventionalCommit {
                message: ":chart_with_upwards_trend: add or update analytics or track code"
                    .to_string(),
            },
            ConventionalCommit {
                message: ":heavy_plus_sign: add a dependency".to_string(),
            },
            ConventionalCommit {
                message: ":heavy_minus_sign: remove a dependency".to_string(),
            },
            ConventionalCommit {
                message: ":passport_control: work on code related to authorization, roles and permissions".to_string(),
            },
        ];
        let (_temp_dir, mut repository) = repo_init();
        for commit in &commits {
            repository = add_commit(repository, commit.message.to_string());
        }

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: commits,
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert!(
            result.has_same_elements(&expected_result),
            "Result doens't have same elements as expected"
        );
    }

    #[test]
    fn creating_from_only_patch_conventional_commits() {
        // Given
        let commits = vec![
            ConventionalCommit {
                message: ":art: improve structure / format of the code".to_string(),
            },
            ConventionalCommit {
                message: ":ambulance: critical hotfix".to_string(),
            },
            ConventionalCommit {
                message: ":lock: fix security or privacy issues".to_string(),
            },
            ConventionalCommit {
                message: "🐛 fix a bug".to_string(),
            },
            ConventionalCommit {
                message: ":zap: improve performance".to_string(),
            },
            ConventionalCommit {
                message: ":goal_net: catch errors".to_string(),
            },
            ConventionalCommit {
                message: ":alien: update code due to external API changes".to_string(),
            },
            ConventionalCommit {
                message: ":wheelchair: improve accessibility".to_string(),
            },
            ConventionalCommit {
                message: ":speech_balloon: add or update text and literals".to_string(),
            },
            ConventionalCommit {
                message: ":mag: improve SEO".to_string(),
            },
            ConventionalCommit {
                message: ":fire: remove code or files".to_string(),
            },
            ConventionalCommit {
                message: ":white_check_mark: add, update, or pass tests".to_string(),
            },
            ConventionalCommit {
                message: ":closed_lock_with_key: add or update secrets".to_string(),
            },
            ConventionalCommit {
                message: ":rotating_light: fix compiler / linter warnings".to_string(),
            },
            ConventionalCommit {
                message: ":green_heart: fix CI build".to_string(),
            },
            ConventionalCommit {
                message: ":arrow_down: downgrade dependencies".to_string(),
            },
            ConventionalCommit {
                message: ":arrow_up: upgrade dependencies".to_string(),
            },
            ConventionalCommit {
                message: ":pushpin: pin dependencies to specific versions".to_string(),
            },
            ConventionalCommit {
                message: ":construction_worker: add or update CI build system".to_string(),
            },
            ConventionalCommit {
                message: ":recycle: refactor code".to_string(),
            },
            ConventionalCommit {
                message: ":wrench: add or update configuration files".to_string(),
            },
            ConventionalCommit {
                message: ":hammer: add or update development scripts".to_string(),
            },
            ConventionalCommit {
                message: ":globe_with_meridians: internationalization and localization".to_string(),
            },
            ConventionalCommit {
                message: ":package: add or update compiled files or packages".to_string(),
            },
            ConventionalCommit {
                message: ":truck: move or rename resources (e.g.: files, paths, routes".to_string(),
            },
            ConventionalCommit {
                message: ":bento: add or update assets".to_string(),
            },
            ConventionalCommit {
                message: ":card_file_box: perform database related changes".to_string(),
            },
            ConventionalCommit {
                message: ":loud_sound: add or update logs".to_string(),
            },
            ConventionalCommit {
                message: ":mute: remove logs".to_string(),
            },
            ConventionalCommit {
                message: ":building_construction: make architectural changes".to_string(),
            },
            ConventionalCommit {
                message: ":camera_flash: add or update snapshots".to_string(),
            },
            ConventionalCommit {
                message: ":label: add or update types".to_string(),
            },
            ConventionalCommit {
                message: ":seedling: add or update seed files".to_string(),
            },
            ConventionalCommit {
                message: ":triangular_flag_on_post: add, update, or remove feature flags"
                    .to_string(),
            },
            ConventionalCommit {
                message: ":dizzy: add or update animations an transitions".to_string(),
            },
            ConventionalCommit {
                message: ":adhesive_bandage: simple fix for a non critical issue".to_string(),
            },
            ConventionalCommit {
                message: ":monocle_face: data exploration / inspection".to_string(),
            },
            ConventionalCommit {
                message: ":necktie: add or update business logic".to_string(),
            },
            ConventionalCommit {
                message: ":stethoscope: add or update healthcheck".to_string(),
            },
            ConventionalCommit {
                message: ":technologist: improve developer experience".to_string(),
            },
            ConventionalCommit {
                message: ":thread: add or update code related to multithreading or concurrency"
                    .to_string(),
            },
            ConventionalCommit {
                message: ":safety_vest: add or update code related to validation".to_string(),
            },
        ];
        let (_temp_dir, mut repository) = repo_init();
        for commit in &commits {
            repository = add_commit(repository, commit.message.to_string());
        }

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: commits,
            other: Vec::new(),
        };
        assert!(
            result.has_same_elements(&expected_result),
            "Result doens't have same elements as expected"
        );
    }

    #[test]
    fn creating_from_only_other_conventional_commits() {
        let commits = vec![
            ConventionalCommit {
                message: ":memo: add or update documentation".to_string(),
            },
            ConventionalCommit {
                message: ":rocket: deploy stuff".to_string(),
            },
            ConventionalCommit {
                message: ":tada: begin a project".to_string(),
            },
            ConventionalCommit {
                message: ":bookmark: release / version tags".to_string(),
            },
            ConventionalCommit {
                message: ":construction: work in progress".to_string(),
            },
            ConventionalCommit {
                message: ":pencil2: fix typos".to_string(),
            },
            ConventionalCommit {
                message: ":poop: write bad code that needs to be improved".to_string(),
            },
            ConventionalCommit {
                message: ":rewind: revert changes".to_string(),
            },
            ConventionalCommit {
                message: ":twisted_rightwards_arrows: merge branches".to_string(),
            },
            ConventionalCommit {
                message: ":page_facing_up: add or update license".to_string(),
            },
            ConventionalCommit {
                message: ":bulb: add or update comments in source code".to_string(),
            },
            ConventionalCommit {
                message: "🍻 write code drunkenly".to_string(),
            },
            ConventionalCommit {
                message: ":bust_in_silhouette: add or update contributor(s)".to_string(),
            },
            ConventionalCommit {
                message: ":clown_face: mock things".to_string(),
            },
            ConventionalCommit {
                message: ":see_no_evil: add or update a .gitignore file".to_string(),
            },
            ConventionalCommit {
                message: ":alembic: perform experiments".to_string(),
            },
            ConventionalCommit {
                message: ":wastebasket: deprecate code that needs to be cleaned up".to_string(),
            },
            ConventionalCommit {
                message: ":coffin: remove dead code".to_string(),
            },
            ConventionalCommit {
                message: ":test_tube: add a failing test".to_string(),
            },
            ConventionalCommit {
                message: ":bricks: infrastructure related changes".to_string(),
            },
            ConventionalCommit {
                message: ":money_with_wings: add sponsorship or money related infrastructure"
                    .to_string(),
            },
        ];
        let (_temp_dir, mut repository) = repo_init();
        for commit in &commits {
            repository = add_commit(repository, commit.message.to_string());
        }

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: commits,
        };
        assert!(
            result.has_same_elements(&expected_result),
            "Result doens't have same elements as expected"
        );
    }
}

#[cfg(test)]
mod evaluate_changes {
    use crate::{Changes, ConventionalCommit, SemanticVersionAction};

    #[test]
    fn has_no_changes() {
        // Given
        let changes = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: vec![ConventionalCommit {
                message: "other commit".to_string(),
            }],
        };

        // When
        let result = changes.define_action_for_semantic_version();

        // Then
        assert_eq!(result, SemanticVersionAction::Keep);
    }

    #[test]
    fn has_patch_changes() {
        // Given
        let changes = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: vec![ConventionalCommit {
                message: "patch commit".to_string(),
            }],
            other: vec![ConventionalCommit {
                message: "other commit".to_string(),
            }],
        };

        // When
        let result = changes.define_action_for_semantic_version();

        // Then
        assert_eq!(result, SemanticVersionAction::IncrementPatch);
    }

    #[test]
    fn has_minor_changes() {
        // Given
        let changes = Changes {
            major: Vec::new(),
            minor: vec![ConventionalCommit {
                message: "minor commit".to_string(),
            }],
            patch: vec![ConventionalCommit {
                message: "patch commit".to_string(),
            }],
            other: vec![ConventionalCommit {
                message: "other commit".to_string(),
            }],
        };

        // When
        let result = changes.define_action_for_semantic_version();

        // Then
        assert_eq!(result, SemanticVersionAction::IncrementMinor);
    }

    #[test]
    fn has_major_changes() {
        // Given
        let changes = Changes {
            major: vec![ConventionalCommit {
                message: "major commit".to_string(),
            }],
            minor: vec![ConventionalCommit {
                message: "minor commit".to_string(),
            }],
            patch: vec![ConventionalCommit {
                message: "patch commit".to_string(),
            }],
            other: vec![ConventionalCommit {
                message: "other commit".to_string(),
            }],
        };

        // When
        let result = changes.define_action_for_semantic_version();

        // Then
        assert_eq!(result, SemanticVersionAction::IncrementMajor);
    }
}
