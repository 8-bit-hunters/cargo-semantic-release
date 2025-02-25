use crate::repo::ConventionalCommit;
use std::collections::HashSet;
use std::fmt::Display;

pub use crate::repo::RepositoryExtension;

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
    /// change categories according to their commit intentions.
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
    pub fn from_repo(repository: &impl RepositoryExtension) -> Self {
        let major_intentions = [(":boom:", "💥")];
        let minor_intentions = [
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
        let patch_intentions = [
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
        let other_intentions = [
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

        let version_tag = repository.get_latest_version_tag().ok().unwrap_or(None);

        let unsorted_commits = match version_tag {
            Some(version_tag) => repository.fetch_commits_until(version_tag.commit_oid),
            None => repository.fetch_all_commits(),
        };

        match unsorted_commits {
            Ok(unsorted_commits) => Self {
                major: get_commits_with_intention(
                    unsorted_commits.clone(),
                    major_intentions.to_vec(),
                ),
                minor: get_commits_with_intention(
                    unsorted_commits.clone(),
                    minor_intentions.to_vec(),
                ),
                patch: get_commits_with_intention(
                    unsorted_commits.clone(),
                    patch_intentions.to_vec(),
                ),
                other: get_commits_with_intention(unsorted_commits, other_intentions.to_vec()),
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

fn get_commits_with_intention(
    commits: Vec<ConventionalCommit>,
    intentions: Vec<(&str, &str)>,
) -> Vec<ConventionalCommit> {
    commits
        .into_iter()
        .filter(|commit| {
            intentions.iter().any(|intention| {
                commit.message.contains(intention.0) || commit.message.contains(intention.1)
            })
        })
        .collect()
}

#[cfg(test)]
mod changes_tests {
    use crate::changes::{Changes, RepositoryExtension};
    use crate::repo::{ConventionalCommit, VersionTag};
    use crate::test_util::MockError;
    use git2::Oid;
    use semver::Version;
    use std::error::Error;

    fn convert(messages: Vec<&str>) -> Vec<ConventionalCommit> {
        messages
            .iter()
            .map(|commit_message| ConventionalCommit {
                message: commit_message.to_string(),
            })
            .collect()
    }

    struct MockedRepository {
        commits: Vec<ConventionalCommit>,
        commit_fetching_fails: bool,
        commit_with_latest_tag: Option<String>,
        latest_version_tag: Option<VersionTag>,
        tag_fetching_fails: bool,
    }

    impl RepositoryExtension for MockedRepository {
        fn fetch_commits_until(
            &self,
            stop_oid: Oid,
        ) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
            assert_eq!(
                stop_oid,
                self.latest_version_tag.as_ref().unwrap().commit_oid,
                "fetch_commits_until is not called with the latest version tag"
            );
            if self.commit_fetching_fails {
                Err(Box::new(MockError))
            } else {
                let commits = self
                    .commits
                    .clone()
                    .into_iter()
                    .rev()
                    .map(|commit| commit.message.clone())
                    .take_while(|message| {
                        message.as_str() != self.commit_with_latest_tag.as_ref().unwrap().as_str()
                    })
                    .map(|message| ConventionalCommit { message })
                    .collect();
                Ok(commits)
            }
        }

        fn fetch_all_commits(&self) -> Result<Vec<ConventionalCommit>, Box<dyn Error>> {
            if self.commit_fetching_fails {
                Err(Box::new(MockError))
            } else {
                Ok(self.commits.clone())
            }
        }

        fn get_latest_version_tag(&self) -> Result<Option<VersionTag>, Box<dyn Error>> {
            if self.tag_fetching_fails {
                Err(Box::new(MockError))
            } else {
                Ok(self.latest_version_tag.clone())
            }
        }
    }

    impl MockedRepository {
        fn from_commits(commits: Vec<&str>) -> Self {
            Self {
                commits: convert(commits),
                commit_fetching_fails: false,
                commit_with_latest_tag: None,
                latest_version_tag: None,
                tag_fetching_fails: false,
            }
        }

        fn new() -> Self {
            Self {
                commits: Vec::new(),
                commit_fetching_fails: false,
                commit_with_latest_tag: None,
                latest_version_tag: None,
                tag_fetching_fails: false,
            }
        }
    }

    #[test]
    fn creating_from_empty_commit_list() {
        // Given
        let repository = MockedRepository::new();

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
    fn error_during_fetching_commits() {
        // Given
        let mut repository = MockedRepository::new();
        repository.commit_fetching_fails = true;

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
        let repository = MockedRepository::from_commits(commit_messages.clone());

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
        let repository = MockedRepository::from_commits(commit_messages.clone());

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
        let repository = MockedRepository::from_commits(commit_messages.clone());

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
        let commit_messages = vec![
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
        let repository = MockedRepository::from_commits(commit_messages.clone());

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: convert(commit_messages),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_repo_with_tags() {
        // Given
        let commit_messages = vec![
            "💥 introduce breaking changes",
            ":sparkles: introduce new feature",
            ":money_with_wings: add sponsorship or money related infrastructure",
            ":memo: add or update documentation",
        ];
        let mut repository = MockedRepository::from_commits(commit_messages.clone());
        repository.latest_version_tag = Some(VersionTag {
            version: Version::new(1, 0, 0),
            commit_oid: Oid::zero(),
        });
        repository.commit_with_latest_tag = Some(commit_messages[1].try_into().unwrap());

        // When
        let result = Changes::from_repo(&repository);

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: convert(commit_messages[2..].to_vec()),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn error_during_fetching_latest_tag() {
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
        let mut repository = MockedRepository::from_commits(commit_messages.clone());
        repository.tag_fetching_fails = true;

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
}

#[cfg(test)]
mod evaluate_changes_tests {
    use crate::changes::{Changes, SemanticVersionAction};
    use crate::repo::ConventionalCommit;

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
