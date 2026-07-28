extern crate cargo_semantic_release;
use cargo_semantic_release::{
    render_tag, Changes, RepositoryExtension, SemanticReleaseConfig, SemanticVersionAction,
};
use clap::Parser;
use clap_cargo::style;
use git2::Repository;
use semver::Version;
use std::{env, process};

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

    /// Increase output verbosity: -v also prints the current version, -vv also prints the
    /// commits since the last version tag
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: SemanticReleaseCommand,
}

#[derive(clap::Subcommand)]
enum SemanticReleaseCommand {
    /// Compute and print the next semantic version derived from commit history
    Version(VersionArgs),
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

    match args.command {
        SemanticReleaseCommand::Version(version_args) => {
            run_version_command(version_args, verbosity)
        }
    }
}

fn run_version_command(args: VersionArgs, verbosity: u8) {
    let path = env::current_dir().unwrap_or_else(|error| {
        eprintln!("Error during getting the current directory:\n\t{error}");
        process::exit(1);
    });

    let config = SemanticReleaseConfig::discover(&path).unwrap_or_else(|error| {
        eprintln!("Error during reading semantic-release config:\n\t{error}");
        process::exit(1);
    });

    let git_repo = Repository::open(&path).unwrap_or_else(|error| {
        eprintln!("Error during opening repository:\n\t{error}");
        process::exit(1);
    });

    if verbosity >= 1 {
        let current_version = current_version(&git_repo, &config).unwrap_or_else(|error| {
            eprintln!("Error during fetching the current version:\n\t{error}");
            process::exit(1);
        });
        println!("Current version: {current_version}");
    }

    if verbosity >= 2 {
        let changes = Changes::from_repo(&git_repo, &config).unwrap_or_else(|error| {
            eprintln!("Error during fetching changes from repository:\n\t{error}");
            process::exit(1);
        });
        println!("Commits since the last version tag:\n{changes}");
    }

    let version = next_version(&git_repo, &config, args.forced_action()).unwrap_or_else(|error| {
        eprintln!("Error during computing the next version:\n\t{error}");
        process::exit(1);
    });

    if args.print_tag {
        println!("{}", render_tag(&config.tag_format, &version));
    } else {
        println!("{version}");
    }
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

/// Compute the next semantic version for `repository`, given `config`.
///
/// Combines [`current_version`] with a [`SemanticVersionAction`]. `forced_action`, when given,
/// is used as-is instead of deriving one from the commits since that tag, in which case the
/// repository's commits aren't parsed at all.
fn next_version(
    repository: &impl RepositoryExtension,
    config: &SemanticReleaseConfig,
    forced_action: Option<SemanticVersionAction>,
) -> Result<Version, Box<dyn std::error::Error>> {
    let action = match forced_action {
        Some(action) => action,
        None => Changes::from_repo(repository, config)?.define_action_for_semantic_version(),
    };

    Ok(action.apply(&current_version(repository, config)?))
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

#[cfg(test)]
mod next_version_tests {
    use crate::next_version;
    use cargo_semantic_release::test_util::{repo_init, RepositoryTestExtensions};
    use cargo_semantic_release::{SemanticReleaseConfig, SemanticVersionAction};
    use semver::Version;

    #[test]
    fn without_a_version_tag_starts_from_0_0_0() {
        // Given
        let commit_messages = vec![":boom: introduce breaking change"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));

        // When
        let result = next_version(&repository, &SemanticReleaseConfig::default(), None);

        // Then
        assert_eq!(result.unwrap(), Version::new(1, 0, 0));
    }

    #[test]
    fn increments_from_the_latest_version_tag() {
        // Given
        let commit_messages = vec![":tada: initial release", ":bug: fix a bug"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let tagged_commit = repository
            .find_commit_by_message(":tada: initial release")
            .unwrap();
        repository.add_tag(tagged_commit, "v1.2.3");

        // When
        let result = next_version(&repository, &SemanticReleaseConfig::default(), None);

        // Then
        assert_eq!(result.unwrap(), Version::new(1, 2, 4));
    }

    #[test]
    fn forced_action_overrides_the_commit_derived_one() {
        // Given
        let commit_messages = vec![":tada: initial release", ":bug: fix a bug"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let tagged_commit = repository
            .find_commit_by_message(":tada: initial release")
            .unwrap();
        repository.add_tag(tagged_commit, "v1.2.3");

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            Some(SemanticVersionAction::IncrementMajor),
        );

        // Then
        assert_eq!(result.unwrap(), Version::new(2, 0, 0));
    }

    #[test]
    fn forced_action_does_not_require_any_commits_to_parse() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            Some(SemanticVersionAction::IncrementPatch),
        );

        // Then
        assert_eq!(result.unwrap(), Version::new(0, 0, 1));
    }
}
