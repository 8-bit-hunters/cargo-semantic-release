extern crate cargo_semantic_release;
use cargo_semantic_release::{
    render_tag, RepositoryExtension, SemanticReleaseConfig, SemanticVersionAction,
};
use clap::Parser;
use clap_cargo::style;
use git2::Oid;
use semver::Version;

mod command;
mod undo_state;
mod version;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
enum CargoCli {
    SemanticRelease(SemanticReleaseArgs),
}

#[derive(clap::Args)]
#[command(version, about, display_name = "semantic-release")]
struct SemanticReleaseArgs {
    /// Run without making any changes: no files written, no commits, tags, or pushes made
    #[arg(long, global = true)]
    noop: bool,

    /// Increase output verbosity: -vv also prints the commits since the last version tag
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: SemanticReleaseCommand,
}

#[derive(clap::Subcommand)]
enum SemanticReleaseCommand {
    /// Compute and print the next semantic version derived from commit history
    Version(VersionArgs),
    /// Undo the changes made by the last `version` run
    Undo(UndoArgs),
}

#[derive(clap::Args)]
struct UndoArgs {
    /// Undo even if HEAD has moved since the version-bump commit was created
    #[arg(long)]
    force: bool,
}

#[derive(clap::Args)]
struct VersionArgs {
    /// Print the next version's tag (e.g. `v1.2.3`) instead of the bare version
    #[arg(long)]
    print_tag: bool,

    /// Force a major version bump instead of deriving it from commit history
    #[arg(long, conflicts_with_all = ["minor", "patch"])]
    major: bool,

    /// Force a minor version bump instead of deriving it from commit history
    #[arg(long, conflicts_with = "patch")]
    minor: bool,

    /// Force a patch version bump instead of deriving it from commit history
    #[arg(long)]
    patch: bool,

    /// Skip creating a commit for the version bump
    #[arg(long)]
    no_commit: bool,

    /// Skip pushing the version-bump commit and any created tags to origin
    #[arg(long)]
    no_push: bool,
}

impl VersionArgs {
    /// The [`SemanticVersionAction`] forced by `--major`/`--minor`/`--patch`, if any.
    fn forced_action(&self) -> Option<SemanticVersionAction> {
        if self.major {
            Some(SemanticVersionAction::IncrementMajor)
        } else if self.minor {
            Some(SemanticVersionAction::IncrementMinor)
        } else if self.patch {
            Some(SemanticVersionAction::IncrementPatch)
        } else {
            None
        }
    }
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(style::HEADER)
    .usage(style::USAGE)
    .literal(style::LITERAL)
    .placeholder(style::PLACEHOLDER)
    .error(style::ERROR)
    .valid(style::VALID)
    .invalid(style::INVALID);

fn main() {
    let CargoCli::SemanticRelease(args) = CargoCli::parse();

    if args.noop {
        println!(
            "Running in no-operation mode (--noop): no files will be written, and no commits, \
             tags, or pushes will be made."
        );
    }

    let verbosity = args.verbose;
    let noop = args.noop;

    match args.command {
        SemanticReleaseCommand::Version(version_args) => {
            command::version::run_version_command(version_args, verbosity, noop)
        }
        SemanticReleaseCommand::Undo(undo_args) => command::undo::run_undo_command(undo_args, noop),
    }
}

/// Whether the commits since the last version tag should be printed, given `verbosity`.
///
/// Requires `-vv` (or higher); `-v` alone does not show the commit log.
fn should_print_commit_log(verbosity: u8) -> bool {
    verbosity >= 2
}

/// Whether the `Cargo.toml` version should be printed, given `verbosity`.
///
/// Requires `-v` (or higher).
fn should_print_cargo_toml_version(verbosity: u8) -> bool {
    verbosity >= 1
}

/// Whether the found tags should be printed, given `verbosity`.
///
/// Requires `-v` (or higher).
fn should_print_found_tags(verbosity: u8) -> bool {
    verbosity >= 1
}

/// Whether the latest tag version should be printed, given `verbosity`.
///
/// Requires `-v` (or higher).
fn should_print_latest_tags_version(verbosity: u8) -> bool {
    verbosity >= 1
}

/// The repository's current version: the latest version tag's version, or `0.0.0` if there is
/// none yet.
fn current_version(
    repository: &impl RepositoryExtension,
    config: &SemanticReleaseConfig,
) -> Result<Version, Box<dyn std::error::Error>> {
    Ok(repository
        .get_latest_version_tag(&config.tag_format)?
        .map(|tag| tag.version)
        .unwrap_or_else(|| Version::new(0, 0, 0)))
}

/// Create the release tag for `version` at `commit_oid`, unless it's already covered by a tag
/// found in `found_tag_names` or by this run's `catch_up_tag`.
///
/// ## Returns
///
/// The tag's name if one was created, `None` if `version` was already tagged.
fn create_release_tag(
    repository: &impl RepositoryExtension,
    tag_format: &str,
    version: &Version,
    commit_oid: Oid,
    found_tag_names: &[String],
    catch_up_tag: &Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let tag_name = render_tag(tag_format, version);
    let already_tagged = catch_up_tag.as_deref() == Some(tag_name.as_str())
        || found_tag_names.iter().any(|name| name == &tag_name);
    if already_tagged {
        return Ok(None);
    }
    repository.create_tag(&tag_name, commit_oid)?;
    Ok(Some(tag_name))
}

#[cfg(test)]
mod create_release_tag_tests {
    use crate::create_release_tag;
    use cargo_semantic_release::test_util::repo_init;
    use cargo_semantic_release::RepositoryExtension;
    use semver::Version;

    #[test]
    fn creates_a_tag_at_the_given_commit_when_none_exists_for_the_version() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let commit_oid = repository.head_commit_oid().unwrap();

        // When
        let result = create_release_tag(
            &repository,
            "v{version}",
            &Version::new(1, 1, 0),
            commit_oid,
            &[],
            &None,
        );

        // Then
        assert_eq!(result.unwrap(), Some("v1.1.0".to_string()));
        let tags = repository.get_all_version_tags("v{version}").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].version, Version::new(1, 1, 0));
    }

    #[test]
    fn skips_when_the_version_is_already_in_found_tags() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let commit_oid = repository.head_commit_oid().unwrap();

        // When
        let result = create_release_tag(
            &repository,
            "v{version}",
            &Version::new(1, 1, 0),
            commit_oid,
            &["v1.1.0".to_string()],
            &None,
        );

        // Then
        assert_eq!(result.unwrap(), None);
        assert!(repository
            .get_all_version_tags("v{version}")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn skips_when_the_version_matches_this_run_s_catch_up_tag() {
        // Given
        let (_temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let commit_oid = repository.head_commit_oid().unwrap();

        // When
        let result = create_release_tag(
            &repository,
            "v{version}",
            &Version::new(1, 1, 0),
            commit_oid,
            &[],
            &Some("v1.1.0".to_string()),
        );

        // Then
        assert_eq!(result.unwrap(), None);
        assert!(repository
            .get_all_version_tags("v{version}")
            .unwrap()
            .is_empty());
    }
}

#[cfg(test)]
mod should_print_commit_log_tests {
    use crate::should_print_commit_log;

    #[test]
    fn is_false_below_double_verbose() {
        assert!(!should_print_commit_log(0));
        assert!(!should_print_commit_log(1));
    }

    #[test]
    fn is_true_at_double_verbose_or_more() {
        assert!(should_print_commit_log(2));
        assert!(should_print_commit_log(3));
    }
}

#[cfg(test)]
mod should_print_cargo_toml_version_tests {
    use crate::should_print_cargo_toml_version;

    #[test]
    fn is_false_without_verbose() {
        assert!(!should_print_cargo_toml_version(0));
    }

    #[test]
    fn is_true_at_verbose_or_more() {
        assert!(should_print_cargo_toml_version(1));
        assert!(should_print_cargo_toml_version(2));
    }
}

#[cfg(test)]
mod should_print_found_tags_tests {
    use crate::should_print_found_tags;

    #[test]
    fn is_false_without_verbose() {
        assert!(!should_print_found_tags(0));
    }

    #[test]
    fn is_true_at_verbose_or_more() {
        assert!(should_print_found_tags(1));
        assert!(should_print_found_tags(2));
    }
}

#[cfg(test)]
mod should_print_latest_tags_version_tests {
    use crate::should_print_latest_tags_version;

    #[test]
    fn is_false_without_verbose() {
        assert!(!should_print_latest_tags_version(0));
    }

    #[test]
    fn is_true_at_verbose_or_more() {
        assert!(should_print_latest_tags_version(1));
        assert!(should_print_latest_tags_version(2));
    }
}

#[cfg(test)]
mod current_version_tests {
    use crate::current_version;
    use cargo_semantic_release::test_util::{repo_init, RepositoryTestExtensions};
    use cargo_semantic_release::SemanticReleaseConfig;
    use semver::Version;

    #[test]
    fn defaults_to_0_0_0_without_a_version_tag() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

        // When
        let result = current_version(&repository, &SemanticReleaseConfig::default());

        // Then
        assert_eq!(result.unwrap(), Version::new(0, 0, 0));
    }

    #[test]
    fn reads_the_version_from_the_latest_version_tag() {
        // Given
        let commit_message = ":tada: initial release";
        let (_temp_dir, repository) = repo_init(Some(vec![commit_message]));
        let tagged_commit = repository.find_commit_by_message(commit_message).unwrap();
        repository.add_tag(tagged_commit, "v1.2.3");

        // When
        let result = current_version(&repository, &SemanticReleaseConfig::default());

        // Then
        assert_eq!(result.unwrap(), Version::new(1, 2, 3));
    }
}
