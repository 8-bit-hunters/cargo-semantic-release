use crate::repo::prelude::*;
use git2::Repository;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;

/// Structure that represents the changes in a git repository
#[derive(Debug)]
pub struct Changes<T: CommitInterface + Clone + Display> {
    /// Vector of commits with major changes
    major: Vec<T>,
    /// Vector of commits with minor changes
    minor: Vec<T>,
    /// Vector of commits with patch changes
    patch: Vec<T>,
    /// Vector of commits with other changes
    other: Vec<T>,
}

impl Changes<GitmojiCommit> {
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
    /// let changes = Changes::from_repo(&git_repo).expect("error during fetching changes");
    /// println!("changes: {changes}")
    /// ```
    pub fn from_repo(repository: &impl RepositoryExtension) -> Result<Self, Box<dyn Error>> {
        let major_intentions = [Gitmoji::Boom];
        let minor_intentions = [
            Gitmoji::Sparkles,
            Gitmoji::ChildrenCrossing,
            Gitmoji::Lipstick,
            Gitmoji::Iphone,
            Gitmoji::Egg,
            Gitmoji::ChartWithUpwardsTrend,
            Gitmoji::HeavyPlusSign,
            Gitmoji::HeavyMinusSign,
            Gitmoji::PassportControl,
        ];
        let patch_intentions = [
            Gitmoji::Art,
            Gitmoji::Ambulance,
            Gitmoji::Lock,
            Gitmoji::Bug,
            Gitmoji::Zap,
            Gitmoji::GoalNet,
            Gitmoji::Alien,
            Gitmoji::Wheelchair,
            Gitmoji::SpeechBalloon,
            Gitmoji::Mag,
            Gitmoji::Fire,
            Gitmoji::WhiteCheckMark,
            Gitmoji::ClosedLockWithKey,
            Gitmoji::RotatingLight,
            Gitmoji::GreenHeart,
            Gitmoji::ArrowDown,
            Gitmoji::ArrowUp,
            Gitmoji::Pushpin,
            Gitmoji::ConstructionWorker,
            Gitmoji::Recycle,
            Gitmoji::Wrench,
            Gitmoji::Hammer,
            Gitmoji::GlobeWithMeridians,
            Gitmoji::Package,
            Gitmoji::Truck,
            Gitmoji::Bento,
            Gitmoji::CardFileBox,
            Gitmoji::LoudSound,
            Gitmoji::Mute,
            Gitmoji::BuildingConstruction,
            Gitmoji::CameraFlash,
            Gitmoji::Label,
            Gitmoji::Seedling,
            Gitmoji::TriangularFlagOnPost,
            Gitmoji::Dizzy,
            Gitmoji::AdhesiveBandage,
            Gitmoji::MonocleFace,
            Gitmoji::Necktie,
            Gitmoji::Stethoscope,
            Gitmoji::Technologist,
            Gitmoji::Thread,
            Gitmoji::SafetyVest,
        ];
        let other_intentions = [
            Gitmoji::Memo,
            Gitmoji::Rocket,
            Gitmoji::Tada,
            Gitmoji::Bookmark,
            Gitmoji::Construction,
            Gitmoji::Pencil2,
            Gitmoji::Poop,
            Gitmoji::Rewind,
            Gitmoji::TwistedRightwardsArrows,
            Gitmoji::PageFacingUp,
            Gitmoji::Bulb,
            Gitmoji::Beers,
            Gitmoji::BustInSilhouette,
            Gitmoji::ClownFace,
            Gitmoji::SeeNoEvil,
            Gitmoji::Alembic,
            Gitmoji::Wastebasket,
            Gitmoji::Coffin,
            Gitmoji::TestTube,
            Gitmoji::Bricks,
            Gitmoji::MoneyWithWings,
        ];

        let version_tag = repository.get_latest_version_tag()?;

        let unsorted_commits = match version_tag {
            Some(version_tag) => repository.fetch_commits_until(version_tag.commit_oid)?,
            None => repository.fetch_all_commits()?,
        };

        let unsorted_commits = unsorted_commits
            .iter()
            .filter_map(|commit| GitmojiCommit::try_from(commit).ok())
            .collect::<Vec<GitmojiCommit>>();

        let major = get_commits_with_intention::<GitmojiCommit>(
            unsorted_commits.clone(),
            major_intentions.to_vec(),
        );

        let minor = get_commits_with_intention::<GitmojiCommit>(
            unsorted_commits.clone(),
            minor_intentions.to_vec(),
        );

        let patch = get_commits_with_intention::<GitmojiCommit>(
            unsorted_commits.clone(),
            patch_intentions.to_vec(),
        );

        let other = get_commits_with_intention::<GitmojiCommit>(
            unsorted_commits.clone(),
            other_intentions.to_vec(),
        );

        Ok(Self {
            major,
            minor,
            patch,
            other,
        })
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
    ///  let action = Changes::from_repo(&git_repo).expect("Error during fetching changes").define_action_for_semantic_version();
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

impl TryFrom<&Repository> for Changes<GitmojiCommit> {
    type Error = Box<dyn Error>;

    fn try_from(value: &Repository) -> Result<Self, Self::Error> {
        Self::from_repo(value)
    }
}

impl PartialEq for Changes<GitmojiCommit> {
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
    /// let changes_1 = Changes::from_repo(&git_repo).expect("error during fetching changes");
    /// let changes_2 = Changes::from_repo(&git_repo).expect("error during fetching changes");
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

impl<T: CommitInterface + Clone + Display> Display for Changes<T> {
    /// Format the values in [`Changes`]
    ///
    /// Example output:
    /// ```shell
    /// major:
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

fn convert_to_string_vector<T: Display>(commits: Vec<T>) -> Vec<String> {
    commits
        .into_iter()
        .map(|commit| format!("{commit}\n"))
        .collect::<Vec<String>>()
}

fn get_commits_with_intention<U>(commits: Vec<U>, intentions: Vec<Gitmoji>) -> Vec<U>
where
    U: CommitInterface,
{
    commits
        .into_iter()
        .filter(|commit| {
            intentions
                .iter()
                .any(|intention| commit.intention() == intention)
        })
        .collect()
}

#[cfg(test)]
mod changes_tests {
    use crate::changes::{Changes, RepositoryExtension};
    use crate::repo::prelude::VersionTag;
    use crate::repo::prelude::*;
    use crate::test_util::{repo_init, MockError, RepositoryTestExtensions};
    use git2::Oid;
    use semver::Version;
    use std::error::Error;

    struct MockedRepository {
        commits: Vec<GitmojiCommit>,
        commit_fetching_fails: bool,
        commit_with_latest_tag: Option<GitmojiCommit>,
        latest_version_tag: Option<VersionTag>,
        tag_fetching_fails: bool,
    }

    impl RepositoryExtension for MockedRepository {
        fn fetch_commits_until(&self, stop_oid: Oid) -> Result<Vec<Commit>, Box<dyn Error>> {
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
                    .take_while(|commit| {
                        commit
                            != self
                                .commit_with_latest_tag
                                .as_ref()
                                .expect("commit for latest tag is not set")
                    })
                    .map(|commit| Commit {
                        message: format!("{} {}", commit.intention(), commit.message()),
                        hash: CommitInterface::hash(&commit).to_string(),
                    })
                    .collect();
                Ok(commits)
            }
        }

        fn fetch_all_commits(&self) -> Result<Vec<Commit>, Box<dyn Error>> {
            if self.commit_fetching_fails {
                Err(Box::new(MockError))
            } else {
                let commits = self
                    .commits
                    .clone()
                    .iter()
                    .map(|commit| Commit {
                        message: format!("{} {}", commit.intention(), commit.message()),
                        hash: CommitInterface::hash(commit).to_string(),
                    })
                    .collect::<Vec<Commit>>();
                Ok(commits)
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
        fn from_commits(commits: Vec<GitmojiCommit>) -> Self {
            Self {
                commits,
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
        let result = Changes::from_repo(&repository).unwrap();

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
        assert!(result.is_err(), "Expected error, but got Ok");
    }

    #[test]
    fn creating_from_only_major_conventional_commits() {
        // Given
        let commit_messages = vec![GitmojiCommit::new(
            "introduce breaking changes".to_string(),
            "".to_string(),
            Gitmoji::Boom,
            "".to_string(),
        )];
        let repository = MockedRepository::from_commits(commit_messages.clone());

        // When
        let result = Changes::from_repo(&repository).unwrap();

        // Then
        let expected_result = Changes {
            major: commit_messages,
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
            GitmojiCommit::new(
                "introduce new feature".to_string(),
                "".to_string(),
                Gitmoji::Sparkles,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "improve user experience / usability".to_string(),
                "".to_string(),
                Gitmoji::ChildrenCrossing,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update the UI and style files".to_string(),
                "".to_string(),
                Gitmoji::Lipstick,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "work on responsive design".to_string(),
                "".to_string(),
                Gitmoji::Iphone,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update an easter egg".to_string(),
                "".to_string(),
                Gitmoji::Egg,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update analytics or track code".to_string(),
                "".to_string(),
                Gitmoji::ChartWithUpwardsTrend,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add a dependency".to_string(),
                "".to_string(),
                Gitmoji::HeavyPlusSign,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "remove a dependency".to_string(),
                "".to_string(),
                Gitmoji::HeavyMinusSign,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "work on code related to authorization, roles and permissions".to_string(),
                "".to_string(),
                Gitmoji::PassportControl,
                "".to_string(),
            ),
        ];
        let repository = MockedRepository::from_commits(commit_messages.clone());

        // When
        let result = Changes::from_repo(&repository).unwrap();

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: commit_messages,
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_patch_conventional_commits() {
        // Given
        let commit_messages = vec![
            GitmojiCommit::new(
                "improve structure / format of the code".to_string(),
                "".to_string(),
                Gitmoji::Art,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "critical hotfix".to_string(),
                "".to_string(),
                Gitmoji::Ambulance,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "fix security or privacy issues".to_string(),
                "".to_string(),
                Gitmoji::Lock,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "fix a bug".to_string(),
                "".to_string(),
                Gitmoji::Bug,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "improve performance".to_string(),
                "".to_string(),
                Gitmoji::Zap,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "catch errors".to_string(),
                "".to_string(),
                Gitmoji::GoalNet,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "update code due to external API changes".to_string(),
                "".to_string(),
                Gitmoji::Alien,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "improve accessibility".to_string(),
                "".to_string(),
                Gitmoji::Wheelchair,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update text and literals".to_string(),
                "".to_string(),
                Gitmoji::SpeechBalloon,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "improve SEO".to_string(),
                "".to_string(),
                Gitmoji::Mag,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "remove code or files".to_string(),
                "".to_string(),
                Gitmoji::Fire,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add, update, or pass tests".to_string(),
                "".to_string(),
                Gitmoji::WhiteCheckMark,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update secrets".to_string(),
                "".to_string(),
                Gitmoji::ClosedLockWithKey,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "fix compiler / linter warnings".to_string(),
                "".to_string(),
                Gitmoji::RotatingLight,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "fix CI build".to_string(),
                "".to_string(),
                Gitmoji::GreenHeart,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "downgrade dependencies".to_string(),
                "".to_string(),
                Gitmoji::ArrowDown,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "upgrade dependencies".to_string(),
                "".to_string(),
                Gitmoji::ArrowUp,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "pin dependencies to specific versions".to_string(),
                "".to_string(),
                Gitmoji::Pushpin,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update CI build system".to_string(),
                "".to_string(),
                Gitmoji::ConstructionWorker,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "refactor code".to_string(),
                "".to_string(),
                Gitmoji::Recycle,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update configuration files".to_string(),
                "".to_string(),
                Gitmoji::Wrench,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update development scripts".to_string(),
                "".to_string(),
                Gitmoji::Hammer,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "internationalization and localization".to_string(),
                "".to_string(),
                Gitmoji::GlobeWithMeridians,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update compiled files or packages".to_string(),
                "".to_string(),
                Gitmoji::Package,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "move or rename resources (e.g.: files, paths, routes".to_string(),
                "".to_string(),
                Gitmoji::Truck,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update assets".to_string(),
                "".to_string(),
                Gitmoji::Bento,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "perform database related changes".to_string(),
                "".to_string(),
                Gitmoji::CardFileBox,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update logs".to_string(),
                "".to_string(),
                Gitmoji::LoudSound,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "remove logs".to_string(),
                "".to_string(),
                Gitmoji::Mute,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "make architectural changes".to_string(),
                "".to_string(),
                Gitmoji::BuildingConstruction,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update snapshots".to_string(),
                "".to_string(),
                Gitmoji::CameraFlash,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update types".to_string(),
                "".to_string(),
                Gitmoji::Label,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update seed files".to_string(),
                "".to_string(),
                Gitmoji::Seedling,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add, update, or remove feature flags".to_string(),
                "".to_string(),
                Gitmoji::TriangularFlagOnPost,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update animations an transitions".to_string(),
                "".to_string(),
                Gitmoji::Dizzy,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "simple fix for a non critical issue".to_string(),
                "".to_string(),
                Gitmoji::AdhesiveBandage,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "data exploration / inspection".to_string(),
                "".to_string(),
                Gitmoji::MonocleFace,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update business logic".to_string(),
                "".to_string(),
                Gitmoji::Necktie,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update healthcheck".to_string(),
                "".to_string(),
                Gitmoji::Stethoscope,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "improve developer experience".to_string(),
                "".to_string(),
                Gitmoji::Technologist,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update code related to multithreading or concurrency".to_string(),
                "".to_string(),
                Gitmoji::Thread,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update code related to validation".to_string(),
                "".to_string(),
                Gitmoji::SafetyVest,
                "".to_string(),
            ),
        ];
        let repository = MockedRepository::from_commits(commit_messages.clone());

        // When
        let result = Changes::from_repo(&repository).unwrap();

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: commit_messages,
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_only_other_conventional_commits() {
        let commit_messages = vec![
            GitmojiCommit::new(
                "add or update documentation".to_string(),
                "".to_string(),
                Gitmoji::Memo,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "deploy stuff".to_string(),
                "".to_string(),
                Gitmoji::Rocket,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "begin a project".to_string(),
                "".to_string(),
                Gitmoji::Tada,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "release / version tags".to_string(),
                "".to_string(),
                Gitmoji::Bookmark,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "work in progress".to_string(),
                "".to_string(),
                Gitmoji::Construction,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "fix typos".to_string(),
                "".to_string(),
                Gitmoji::Pencil2,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "write bad code that needs to be improved".to_string(),
                "".to_string(),
                Gitmoji::Poop,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "revert changes".to_string(),
                "".to_string(),
                Gitmoji::Rewind,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "merge branches".to_string(),
                "".to_string(),
                Gitmoji::TwistedRightwardsArrows,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update license".to_string(),
                "".to_string(),
                Gitmoji::PageFacingUp,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update comments in source code".to_string(),
                "".to_string(),
                Gitmoji::Bulb,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "write code drunkenly".to_string(),
                "".to_string(),
                Gitmoji::Beers,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update contributor(s)".to_string(),
                "".to_string(),
                Gitmoji::BustInSilhouette,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "mock things".to_string(),
                "".to_string(),
                Gitmoji::ClownFace,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update a .gitignore file".to_string(),
                "".to_string(),
                Gitmoji::SeeNoEvil,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "perform experiments".to_string(),
                "".to_string(),
                Gitmoji::Alembic,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "deprecate code that needs to be cleaned up".to_string(),
                "".to_string(),
                Gitmoji::Wastebasket,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "remove dead code".to_string(),
                "".to_string(),
                Gitmoji::Coffin,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add a failing test".to_string(),
                "".to_string(),
                Gitmoji::TestTube,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "infrastructure related changes".to_string(),
                "".to_string(),
                Gitmoji::Bricks,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add sponsorship or money related infrastructure".to_string(),
                "".to_string(),
                Gitmoji::MoneyWithWings,
                "".to_string(),
            ),
        ];
        let repository = MockedRepository::from_commits(commit_messages.clone());

        // When
        let result = Changes::from_repo(&repository).unwrap();

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: commit_messages,
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn creating_from_repo_with_tags() {
        // Given
        let commit_messages = vec![
            GitmojiCommit::new(
                "introduce breaking changes".to_string(),
                "".to_string(),
                Gitmoji::Boom,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "introduce new feature".to_string(),
                "".to_string(),
                Gitmoji::Sparkles,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add sponsorship or money related infrastructure".to_string(),
                "".to_string(),
                Gitmoji::MoneyWithWings,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update documentation".to_string(),
                "".to_string(),
                Gitmoji::Memo,
                "".to_string(),
            ),
        ];
        let mut repository = MockedRepository::from_commits(commit_messages.clone());
        repository.latest_version_tag = Some(VersionTag {
            version: Version::new(1, 0, 0),
            commit_oid: Oid::zero(),
        });
        repository.commit_with_latest_tag = Some(commit_messages[1].clone());

        // When
        let result = Changes::from_repo(&repository).unwrap();

        // Then
        let expected_result = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: commit_messages[2..].to_vec(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn error_during_fetching_latest_tag() {
        // Given
        let commit_messages = vec![
            GitmojiCommit::new(
                "introduce new feature".to_string(),
                "".to_string(),
                Gitmoji::Sparkles,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "improve user experience / usability".to_string(),
                "".to_string(),
                Gitmoji::ChildrenCrossing,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update the UI and style files".to_string(),
                "".to_string(),
                Gitmoji::Lipstick,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "work on responsive design".to_string(),
                "".to_string(),
                Gitmoji::Iphone,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update an easter egg".to_string(),
                "".to_string(),
                Gitmoji::Egg,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add or update analytics or track code".to_string(),
                "".to_string(),
                Gitmoji::ChartWithUpwardsTrend,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "add a dependency".to_string(),
                "".to_string(),
                Gitmoji::HeavyPlusSign,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "remove a dependency".to_string(),
                "".to_string(),
                Gitmoji::HeavyMinusSign,
                "".to_string(),
            ),
            GitmojiCommit::new(
                "work on code related to authorization, roles and permissions".to_string(),
                "".to_string(),
                Gitmoji::PassportControl,
                "".to_string(),
            ),
        ];
        let mut repository = MockedRepository::from_commits(commit_messages.clone());
        repository.tag_fetching_fails = true;

        // When
        let result = Changes::from_repo(&repository);

        // Then
        assert!(result.is_err(), "Expected Error, got Ok");
    }

    #[test]
    fn creating_with_try_from() {
        // Given
        let commit_messages = vec!["💥 introduce breaking changes"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages.clone()));
        let commit = repository
            .find_commit_by_message("💥 introduce breaking changes")
            .unwrap();

        // When
        let result = Changes::try_from(&repository).unwrap();

        // Then
        let hash = commit.id().to_string();
        let expected_result = Changes {
            major: vec![GitmojiCommit::new(
                "introduce breaking changes".to_string(),
                hash,
                Gitmoji::Boom,
                "".to_string(),
            )],
            minor: Vec::new(),
            patch: Vec::new(),
            other: Vec::new(),
        };
        assert_eq!(result, expected_result);
    }
}

#[cfg(test)]
mod evaluate_changes_tests {
    use crate::changes::{Changes, SemanticVersionAction};
    use crate::repo::prelude::{Gitmoji, GitmojiCommit};
    use Default;

    #[test]
    fn has_no_changes() {
        // Given
        let changes = Changes {
            major: Vec::new(),
            minor: Vec::new(),
            patch: Vec::new(),
            other: vec![GitmojiCommit::new(
                "other".to_string(),
                Default::default(),
                Gitmoji::Memo,
                Default::default(),
            )],
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
            patch: vec![GitmojiCommit::new(
                "patch".to_string(),
                Default::default(),
                Gitmoji::Bug,
                Default::default(),
            )],
            other: vec![GitmojiCommit::new(
                "other".to_string(),
                Default::default(),
                Gitmoji::Memo,
                Default::default(),
            )],
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
            minor: vec![GitmojiCommit::new(
                "minor".to_string(),
                Default::default(),
                Gitmoji::Sparkles,
                Default::default(),
            )],
            patch: vec![GitmojiCommit::new(
                "patch".to_string(),
                Default::default(),
                Gitmoji::Bug,
                Default::default(),
            )],
            other: vec![GitmojiCommit::new(
                "other".to_string(),
                Default::default(),
                Gitmoji::Memo,
                Default::default(),
            )],
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
            major: vec![GitmojiCommit::new(
                "major".to_string(),
                Default::default(),
                Gitmoji::Boom,
                Default::default(),
            )],
            minor: vec![GitmojiCommit::new(
                "minor".to_string(),
                Default::default(),
                Gitmoji::Sparkles,
                Default::default(),
            )],
            patch: vec![GitmojiCommit::new(
                "patch".to_string(),
                Default::default(),
                Gitmoji::Bug,
                Default::default(),
            )],
            other: vec![GitmojiCommit::new(
                "other".to_string(),
                Default::default(),
                Gitmoji::Memo,
                Default::default(),
            )],
        };

        // When
        let result = changes.define_action_for_semantic_version();

        // Then
        assert_eq!(result, SemanticVersionAction::IncrementMajor);
    }
}
