extern crate cargo_semantic_release;
use cargo_semantic_release::{
    render_tag, Changes, RepositoryExtension, SemanticReleaseConfig, SemanticVersionAction,
};
use clap::Parser;
use clap_cargo::style;
use git2::{Oid, Repository};
use semver::Version;
use std::path::Path;
use std::{env, process};

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
            run_version_command(version_args, verbosity, noop)
        }
        SemanticReleaseCommand::Undo(undo_args) => run_undo_command(undo_args, noop),
    }
}

fn run_version_command(args: VersionArgs, verbosity: u8, noop: bool) {
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

    let cargo_toml_path = path.join("Cargo.toml");
    let cargo_toml_version = version::get_cargo_version(&cargo_toml_path).unwrap_or_else(|error| {
        eprintln!("Error during reading Cargo.toml:\n\t{error}");
        process::exit(1);
    });
    if should_print_cargo_toml_version(verbosity) {
        println!("Cargo.toml version: {cargo_toml_version}");
    }

    let mut found_tags = git_repo
        .get_all_version_tags(&config.tag_format)
        .unwrap_or_else(|error| {
            eprintln!("Error during fetching version tags:\n\t{error}");
            process::exit(1);
        });
    found_tags.sort();
    let found_tags_display = if found_tags.is_empty() {
        "none".to_string()
    } else {
        found_tags
            .iter()
            .map(|tag| render_tag(&config.tag_format, &tag.version))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if should_print_found_tags(verbosity) {
        println!("Found tags: {found_tags_display}");
    }

    let repo_current_version = current_version(&git_repo, &config).unwrap_or_else(|error| {
        eprintln!("Error during fetching the current version:\n\t{error}");
        process::exit(1);
    });
    if should_print_latest_tags_version(verbosity) {
        println!("Latest tag version: {repo_current_version}");
    }

    if should_print_commit_log(verbosity) {
        let changes = Changes::from_repo(&git_repo, &config).unwrap_or_else(|error| {
            eprintln!("Error during fetching changes from repository:\n\t{error}");
            process::exit(1);
        });
        println!("Commits since the last version tag:\n{changes}");
    }

    let (version, catch_up_tag) = next_version(
        &git_repo,
        &config,
        &cargo_toml_version,
        args.forced_action(),
        noop,
    )
    .unwrap_or_else(|error| {
        eprintln!("Error during computing the next version:\n\t{error}");
        process::exit(1);
    });

    if verbosity >= 1 {
        if let Some(tag_name) = &catch_up_tag {
            println!("Created catch-up tag: {tag_name}");
        }
    }

    if !noop {
        version::set_cargo_version(&cargo_toml_path, &version).unwrap_or_else(|error| {
            eprintln!("Error during writing Cargo.toml:\n\t{error}");
            process::exit(1);
        });

        let commit_oid = if !args.no_commit {
            let commit_message = format!(
                ":bookmark: Bump release version to {}",
                render_tag(&config.tag_format, &version)
            );
            Some(
                git_repo
                    .commit_file(Path::new("Cargo.toml"), &commit_message)
                    .unwrap_or_else(|error| {
                        eprintln!("Error during committing the version bump:\n\t{error}");
                        process::exit(1);
                    }),
            )
        } else {
            None
        };

        let last_run_state = undo_state::LastRunState::new(
            &cargo_toml_version,
            &version,
            commit_oid,
            catch_up_tag.clone(),
        );
        let git_dir = git_repo.path();
        undo_state::write(git_dir, &last_run_state).unwrap_or_else(|error| {
            eprintln!("Error during recording undo state:\n\t{error}");
            process::exit(1);
        });
    }

    if args.print_tag {
        println!("Next version: {}", render_tag(&config.tag_format, &version));
    } else {
        println!("Next version: {version}");
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

fn run_undo_command(args: UndoArgs, noop: bool) {
    let path = env::current_dir().unwrap_or_else(|error| {
        eprintln!("Error during getting the current directory:\n\t{error}");
        process::exit(1);
    });

    let git_repo = Repository::open(&path).unwrap_or_else(|error| {
        eprintln!("Error during opening repository:\n\t{error}");
        process::exit(1);
    });

    let git_dir = git_repo.path().to_path_buf();
    let state = undo_state::read(&git_dir)
        .unwrap_or_else(|error| {
            eprintln!("Error during reading undo state:\n\t{error}");
            process::exit(1);
        })
        .unwrap_or_else(|| {
            eprintln!("No previous `version` run found to undo.");
            process::exit(1);
        });

    if noop {
        println!(
            "Would restore Cargo.toml to version {}.",
            state.previous_version
        );
        if let Some(commit_oid) = &state.commit_oid {
            println!("Would remove the version-bump commit ({commit_oid}).");
        }
        if let Some(tag_name) = &state.catch_up_tag {
            println!("Would remove the catch-up tag {tag_name}.");
        }
        return;
    }

    let cargo_toml_path = path.join("Cargo.toml");
    perform_undo(&git_repo, &cargo_toml_path, &state, args.force).unwrap_or_else(|error| {
        eprintln!("Error during undoing the last version run:\n\t{error}");
        process::exit(1);
    });

    undo_state::delete(&git_dir).unwrap_or_else(|error| {
        eprintln!("Error during clearing undo state:\n\t{error}");
        process::exit(1);
    });

    println!("Restored Cargo.toml to version {}.", state.previous_version);
    if let Some(commit_oid) = &state.commit_oid {
        println!("Removed the version-bump commit ({commit_oid}).");
    }
    if let Some(tag_name) = &state.catch_up_tag {
        println!("Removed the catch-up tag {tag_name}.");
    }
}

/// Reverse the repository-facing effects recorded in `state`: restore `cargo_toml_path`'s
/// declared version and, if a bump commit or catch-up tag were created, undo those too.
///
/// Refuses (returning `Err` without changing anything) if `state` recorded a bump commit and
/// `HEAD` no longer points at it, unless `force` is set.
fn perform_undo(
    repository: &impl RepositoryExtension,
    cargo_toml_path: &Path,
    state: &undo_state::LastRunState,
    force: bool,
) -> Result<(), String> {
    if let Some(commit_oid) = &state.commit_oid {
        let commit_oid = Oid::from_str(commit_oid).map_err(|error| error.to_string())?;
        let head_oid = repository
            .head_commit_oid()
            .map_err(|error| error.to_string())?;
        if commit_oid != head_oid && !force {
            return Err(format!(
                "HEAD has moved since the version-bump commit ({commit_oid}) was created; \
                 refusing to undo automatically. Revert it manually (e.g. `git revert \
                 {commit_oid}`), or re-run with --force."
            ));
        }
    }

    let previous_version =
        Version::parse(&state.previous_version).map_err(|error| error.to_string())?;
    version::set_cargo_version(cargo_toml_path, &previous_version)?;

    if let Some(commit_oid) = &state.commit_oid {
        let commit_oid = Oid::from_str(commit_oid).map_err(|error| error.to_string())?;
        let parent_oid = repository
            .commit_parent_oid(commit_oid)
            .map_err(|error| error.to_string())?;
        repository
            .reset_soft_to(parent_oid)
            .map_err(|error| error.to_string())?;
    }

    if let Some(tag_name) = &state.catch_up_tag {
        repository
            .delete_tag(tag_name)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
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
/// Combines a baseline version with a [`SemanticVersionAction`]. `forced_action`, when given,
/// is used as-is instead of deriving one from the commits since the latest tag, in which case
/// the repository's commits aren't parsed at all.
///
/// The baseline is normally [`current_version`] (the latest tag). But if `cargo_toml_version`
/// (the version currently declared in `Cargo.toml`) is *ahead* of that, the tag history is
/// missing a tag for it, e.g. it was bumped by hand without tagging. In that case a catch-up
/// tag for `cargo_toml_version` is created at `HEAD` (skipped when `noop` is set, though the
/// returned version is computed the same way either way), and it becomes the baseline instead.
///
/// ## Returns
///
/// The next [`Version`], and the name of the catch-up tag if one was created (`None` if `noop`
/// was set or no reconciliation was needed).
fn next_version(
    repository: &impl RepositoryExtension,
    config: &SemanticReleaseConfig,
    cargo_toml_version: &Version,
    forced_action: Option<SemanticVersionAction>,
    noop: bool,
) -> Result<(Version, Option<String>), Box<dyn std::error::Error>> {
    let action = match forced_action {
        Some(action) => action,
        None => Changes::from_repo(repository, config)?.define_action_for_semantic_version(),
    };

    let tag_based_current_version = current_version(repository, config)?;

    let mut catch_up_tag = None;

    let baseline = if cargo_toml_version > &tag_based_current_version {
        let tag_name = render_tag(&config.tag_format, cargo_toml_version);
        if !noop {
            let head_commit_oid = repository.head_commit_oid().map_err(|error| {
                format!("failed to resolve HEAD while creating catch-up tag '{tag_name}': {error}")
            })?;
            repository
                .create_tag(&tag_name, head_commit_oid)
                .map_err(|error| format!("failed to create catch-up tag '{tag_name}': {error}"))?;
            catch_up_tag = Some(tag_name.clone());
        }
        println!(
            "Since Cargo.toml ({cargo_toml_version}) > latest tag version \
            ({tag_based_current_version}), current version is {}",
            tag_name
        );
        cargo_toml_version.clone()
    } else {
        tag_based_current_version
    };

    Ok((action.apply(&baseline), catch_up_tag))
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
mod perform_undo_tests {
    use crate::perform_undo;
    use crate::undo_state::LastRunState;
    use cargo_semantic_release::test_util::{repo_init, RepositoryTestExtensions};
    use cargo_semantic_release::RepositoryExtension;
    use semver::Version;
    use std::path::Path;

    #[test]
    fn restores_cargo_toml_reverts_the_commit_and_removes_the_catch_up_tag() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let pre_bump_head = repository.head_commit_oid().unwrap();
        repository.create_tag("v1.0.0", pre_bump_head).unwrap();

        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.1.0\"\n",
        )
        .unwrap();
        let bump_commit_oid = repository
            .commit_file(Path::new("Cargo.toml"), "bump")
            .unwrap();

        let state = LastRunState::new(
            &Version::new(1, 0, 0),
            &Version::new(1, 1, 0),
            Some(bump_commit_oid),
            Some("v1.0.0".to_string()),
        );

        // When
        let result = perform_undo(&repository, &cargo_toml_path, &state, false);

        // Then
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(repository.head_commit_oid().unwrap(), pre_bump_head);
        assert_eq!(
            std::fs::read_to_string(&cargo_toml_path).unwrap(),
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n"
        );
        assert!(repository
            .get_all_version_tags("v{version}")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn only_restores_cargo_toml_when_no_commit_was_recorded() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.1.0\"\n",
        )
        .unwrap();
        let head_before = repository.head_commit_oid().unwrap();

        let state = LastRunState::new(&Version::new(1, 0, 0), &Version::new(1, 1, 0), None, None);

        // When
        let result = perform_undo(&repository, &cargo_toml_path, &state, false);

        // Then
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(repository.head_commit_oid().unwrap(), head_before);
        assert_eq!(
            std::fs::read_to_string(&cargo_toml_path).unwrap(),
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n"
        );
    }

    #[test]
    fn refuses_when_head_has_moved_past_the_bump_commit_without_force() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let bump_commit_oid = repository
            .commit_file(Path::new("Cargo.toml"), "bump")
            .unwrap();
        repository.add_commit("later commit");

        let state = LastRunState::new(
            &Version::new(1, 0, 0),
            &Version::new(1, 1, 0),
            Some(bump_commit_oid),
            None,
        );

        // When
        let result = perform_undo(&repository, &cargo_toml_path, &state, false);

        // Then
        assert!(result.is_err());
        assert_eq!(
            std::fs::read_to_string(&cargo_toml_path).unwrap(),
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n",
            "Cargo.toml should be untouched when undo refuses"
        );
    }

    #[test]
    fn force_resets_even_when_head_has_moved_past_the_bump_commit() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let pre_bump_head = repository.head_commit_oid().unwrap();
        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.1.0\"\n",
        )
        .unwrap();
        let bump_commit_oid = repository
            .commit_file(Path::new("Cargo.toml"), "bump")
            .unwrap();
        repository.add_commit("later commit");

        let state = LastRunState::new(
            &Version::new(1, 0, 0),
            &Version::new(1, 1, 0),
            Some(bump_commit_oid),
            None,
        );

        // When
        let result = perform_undo(&repository, &cargo_toml_path, &state, true);

        // Then
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(repository.head_commit_oid().unwrap(), pre_bump_head);
        assert_eq!(
            std::fs::read_to_string(&cargo_toml_path).unwrap(),
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n"
        );
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

#[cfg(test)]
mod next_version_tests {
    use crate::next_version;
    use cargo_semantic_release::test_util::{repo_init, RepositoryTestExtensions};
    use cargo_semantic_release::{
        RepositoryExtension, SemanticReleaseConfig, SemanticVersionAction,
    };
    use semver::Version;

    #[test]
    fn without_a_version_tag_starts_from_0_0_0() {
        // Given
        let commit_messages = vec![":boom: introduce breaking change"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            &Version::new(0, 0, 0),
            None,
            false,
        );

        // Then
        assert_eq!(result.unwrap(), (Version::new(1, 0, 0), None));
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
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            &Version::new(0, 0, 0),
            None,
            false,
        );

        // Then
        assert_eq!(result.unwrap(), (Version::new(1, 2, 4), None));
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
            &Version::new(0, 0, 0),
            Some(SemanticVersionAction::IncrementMajor),
            false,
        );

        // Then
        assert_eq!(result.unwrap(), (Version::new(2, 0, 0), None));
    }

    #[test]
    fn forced_action_does_not_require_any_commits_to_parse() {
        // Given
        let (_temp_dir, repository) = repo_init(None);

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            &Version::new(0, 0, 0),
            Some(SemanticVersionAction::IncrementPatch),
            false,
        );

        // Then
        assert_eq!(result.unwrap(), (Version::new(0, 0, 1), None));
    }

    #[test]
    fn does_not_reconcile_when_cargo_toml_version_is_not_ahead_of_the_latest_tag() {
        // Given
        let commit_messages = vec![":tada: initial release", ":bug: fix a bug"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let tagged_commit = repository
            .find_commit_by_message(":tada: initial release")
            .unwrap();
        repository.add_tag(tagged_commit, "v1.0.0");

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            &Version::new(1, 0, 0),
            None,
            false,
        );

        // Then
        assert_eq!(result.unwrap(), (Version::new(1, 0, 1), None));
        assert_eq!(
            repository
                .get_latest_version_tag("v{version}")
                .unwrap()
                .unwrap()
                .version,
            Version::new(1, 0, 0),
            "no catch-up tag should have been created"
        );
    }

    #[test]
    fn reconciles_when_cargo_toml_version_is_ahead_of_the_latest_tag() {
        // Given
        let commit_messages = vec![":tada: initial release", ":bug: fix a bug"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let tagged_commit = repository
            .find_commit_by_message(":tada: initial release")
            .unwrap();
        repository.add_tag(tagged_commit, "v1.0.0");

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            &Version::new(2, 0, 0),
            None,
            false,
        );

        // Then
        let (version, catch_up_tag) = result.unwrap();
        assert_eq!(version, Version::new(2, 0, 1));
        assert_eq!(
            catch_up_tag.as_deref(),
            Some("v2.0.0"),
            "the created catch-up tag's name should have been returned"
        );
        assert_eq!(
            repository
                .get_latest_version_tag("v{version}")
                .unwrap()
                .unwrap()
                .version,
            Version::new(2, 0, 0),
            "a catch-up tag for the Cargo.toml version should have been created"
        );
    }

    #[test]
    fn noop_reconciliation_computes_the_version_without_creating_the_catch_up_tag() {
        // Given
        let commit_messages = vec![":tada: initial release", ":bug: fix a bug"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let tagged_commit = repository
            .find_commit_by_message(":tada: initial release")
            .unwrap();
        repository.add_tag(tagged_commit, "v1.0.0");

        // When
        let result = next_version(
            &repository,
            &SemanticReleaseConfig::default(),
            &Version::new(2, 0, 0),
            None,
            true,
        );

        // Then
        assert_eq!(result.unwrap(), (Version::new(2, 0, 1), None));
        assert_eq!(
            repository
                .get_latest_version_tag("v{version}")
                .unwrap()
                .unwrap()
                .version,
            Version::new(1, 0, 0),
            "--noop should not have created the catch-up tag"
        );
    }
}
