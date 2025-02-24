use crate::conventional_commit::ConventionalCommit;
use git2::Repository;
use std::collections::HashSet;
use std::fmt::Display;

pub use crate::commit_fetcher::RepositoryFetchCommitExtension;

/// Structure that represents the changes in a git repository
#[derive(Debug)]
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

        match repository.fetch_commits_since_last_version() {
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
}

impl PartialEq for Changes {
    /// Compare two [`Changes`] struct to see if they have the same elements.
    ///
    /// # Returns
    ///
    /// `true` if the two structure has the same elements regardless they order, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use git2::AttrValue::True;
    /// use git2::Repository;
    /// use cargo_semantic_release::Changes;
    ///
    /// let git_repo = Repository::open(".").unwrap();
    ///
    /// let changes_1 = Changes::from_repo(&git_repo);
    /// let changes_2 = Changes::from_repo(&git_repo);
    ///
    /// assert_eq!(changes_1, changes_2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
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
    /// Format the values in [`Changes`]
    ///
    /// Example output:
    /// ```shell
    ///major:
    ///         :boom: Introduce breaking change
    ///
    /// minor:
    ///         :sparkles: Add new feature
    ///
    /// patch:
    ///         :recycle: Refactor codebase
    ///
    /// other:
    ///         :bulb: Add comments
    /// ```
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

#[cfg(test)]
mod changes_struct {
    use crate::changes::Changes;
    use crate::conventional_commit::ConventionalCommit;
    use crate::test_util::repo_init;

    fn convert(messages: Vec<&str>) -> Vec<ConventionalCommit> {
        messages
            .iter()
            .map(|commit_message| ConventionalCommit {
                message: commit_message.to_string(),
            })
            .collect()
    }

    #[test]
    fn creating_from_empty_commit_list() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

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
        let commit_messages = vec!["💥 introduce breaking changes"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: convert(commit_messages),
            minor: Vec::new(),
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_minor_conventional_commits() {
        // Given
        let commit_messages = vec![
            ":sparkles: introduce new feature",
            ":children_crossing: improve user experience / usability",
            "💄 add or update the UI and style files",
            ":iphone: work on responsive design",
            ":egg: add or update an easter egg",
            ":chart_with_upwards_trend: add or update analytics or track code",
            ":heavy_plus_sign: add a dependency",
            ":heavy_minus_sign: remove a dependency",
            ":passport_control: work on code related to authorization, roles and permissions",
        ];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: convert(commit_messages),
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_patch_conventional_commits() {
        // Given
        let commit_messages = vec![
            ":art: improve structure / format of the code",
            ":ambulance: critical hotfix",
            ":lock: fix security or privacy issues",
            "🐛 fix a bug",
            ":zap: improve performance",
            ":goal_net: catch errors",
            ":alien: update code due to external API changes",
            ":wheelchair: improve accessibility",
            ":speech_balloon: add or update text and literals",
            ":mag: improve SEO",
            ":fire: remove code or files",
            ":white_check_mark: add, update, or pass tests",
            ":closed_lock_with_key: add or update secrets",
            ":rotating_light: fix compiler / linter warnings",
            ":green_heart: fix CI build",
            ":arrow_down: downgrade dependencies",
            ":arrow_up: upgrade dependencies",
            ":pushpin: pin dependencies to specific versions",
            ":construction_worker: add or update CI build system",
            ":recycle: refactor code",
            ":wrench: add or update configuration files",
            ":hammer: add or update development scripts",
            ":globe_with_meridians: internationalization and localization",
            ":package: add or update compiled files or packages",
            ":truck: move or rename resources (e.g.: files, paths, routes",
            ":bento: add or update assets",
            ":card_file_box: perform database related changes",
            ":loud_sound: add or update logs",
            ":mute: remove logs",
            ":building_construction: make architectural changes",
            ":camera_flash: add or update snapshots",
            ":label: add or update types",
            ":seedling: add or update seed files",
            ":triangular_flag_on_post: add, update, or remove feature flags",
            ":dizzy: add or update animations an transitions",
            ":adhesive_bandage: simple fix for a non critical issue",
            ":monocle_face: data exploration / inspection",
            ":necktie: add or update business logic",
            ":stethoscope: add or update healthcheck",
            ":technologist: improve developer experience",
            ":thread: add or update code related to multithreading or concurrency",
            ":safety_vest: add or update code related to validation",
        ];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: convert(commit_messages),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_other_conventional_commits() {
        let commit_message = vec![
            ":memo: add or update documentation",
            ":rocket: deploy stuff",
            ":tada: begin a project",
            ":bookmark: release / version tags",
            ":construction: work in progress",
            ":pencil2: fix typos",
            ":poop: write bad code that needs to be improved",
            ":rewind: revert changes",
            ":twisted_rightwards_arrows: merge branches",
            ":page_facing_up: add or update license",
            ":bulb: add or update comments in source code",
            "🍻 write code drunkenly",
            ":bust_in_silhouette: add or update contributor(s)",
            ":clown_face: mock things",
            ":see_no_evil: add or update a .gitignore file",
            ":alembic: perform experiments",
            ":wastebasket: deprecate code that needs to be cleaned up",
            ":coffin: remove dead code",
            ":test_tube: add a failing test",
            ":bricks: infrastructure related changes",
            ":money_with_wings: add sponsorship or money related infrastructure",
        ];
        let (_temp_dir, repository) = repo_init(Some(commit_message.clone()));

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: convert(commit_message),
        };
        assert_eq!(result, expected_result);
    }
}

#[cfg(test)]
mod evaluate_changes_tests {
    use crate::changes::{Changes, SemanticVersionAction};
    use crate::conventional_commit::ConventionalCommit;

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
