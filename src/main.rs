extern crate cargo_semantic_release;
use cargo_semantic_release::{Changes, RepositoryExtension, SemanticReleaseConfig};
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
    #[command(subcommand)]
    command: SemanticReleaseCommand,
}

#[derive(clap::Subcommand)]
enum SemanticReleaseCommand {
    /// Compute and print the next semantic version derived from commit history
    Version,
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

    match args.command {
        SemanticReleaseCommand::Version => run_version_command(),
    }
}

fn run_version_command() {
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

    let version = next_version(&git_repo, &config).unwrap_or_else(|error| {
        eprintln!("Error during computing the next version:\n\t{error}");
        process::exit(1);
    });

    println!("{version}");
}

/// Compute the next semantic version for `repository`, given `config`.
///
/// Combines the latest version tag (or `0.0.0` if there is none) with the
/// [`SemanticVersionAction`](cargo_semantic_release::SemanticVersionAction) derived from the
/// commits since that tag.
fn next_version(
    repository: &impl RepositoryExtension,
    config: &SemanticReleaseConfig,
) -> Result<Version, Box<dyn std::error::Error>> {
    let current_version = repository
        .get_latest_version_tag(&config.tag_format)?
        .map(|tag| tag.version)
        .unwrap_or_else(|| Version::new(0, 0, 0));

    let action = Changes::from_repo(repository, config)?.define_action_for_semantic_version();

    Ok(action.apply(&current_version))
}

#[cfg(test)]
mod next_version_tests {
    use crate::next_version;
    use cargo_semantic_release::test_util::{repo_init, RepositoryTestExtensions};
    use cargo_semantic_release::SemanticReleaseConfig;
    use semver::Version;

    #[test]
    fn without_a_version_tag_starts_from_0_0_0() {
        // Given
        let commit_messages = vec![":boom: introduce breaking change"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));

        // When
        let result = next_version(&repository, &SemanticReleaseConfig::default());

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
        let result = next_version(&repository, &SemanticReleaseConfig::default());

        // Then
        assert_eq!(result.unwrap(), Version::new(1, 2, 4));
    }
}
