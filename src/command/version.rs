use crate::VersionArgs;
use cargo_semantic_release::{
    render_tag, Changes, RepositoryExtension, SemanticReleaseConfig, SemanticVersionAction,
};
use git2::Repository;
use semver::Version;
use std::path::Path;
use std::{env, process};

pub fn run_version_command(args: VersionArgs, verbosity: u8, noop: bool) {
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
    let cargo_toml_version =
        crate::version::get_cargo_version(&cargo_toml_path).unwrap_or_else(|error| {
            eprintln!("Error during reading Cargo.toml:\n\t{error}");
            process::exit(1);
        });
    if crate::should_print_cargo_toml_version(verbosity) {
        println!("Cargo.toml version: {cargo_toml_version}");
    }

    let mut found_tags = git_repo
        .get_all_version_tags(&config.tag_format)
        .unwrap_or_else(|error| {
            eprintln!("Error during fetching version tags:\n\t{error}");
            process::exit(1);
        });
    found_tags.sort();
    let found_tag_names: Vec<String> = found_tags
        .iter()
        .map(|tag| render_tag(&config.tag_format, &tag.version))
        .collect();
    let found_tags_display = if found_tag_names.is_empty() {
        "none".to_string()
    } else {
        found_tag_names.join(", ")
    };
    if crate::should_print_found_tags(verbosity) {
        println!("Found tags: {found_tags_display}");
    }

    let repo_current_version = crate::current_version(&git_repo, &config).unwrap_or_else(|error| {
        eprintln!("Error during fetching the current version:\n\t{error}");
        process::exit(1);
    });
    if crate::should_print_latest_tags_version(verbosity) {
        println!("Latest tag version: {repo_current_version}");
    }

    if crate::should_print_commit_log(verbosity) {
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
        crate::version::set_cargo_version(&cargo_toml_path, &version).unwrap_or_else(|error| {
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

        let release_tag = commit_oid.and_then(|commit_oid| {
            crate::create_release_tag(
                &git_repo,
                &config.tag_format,
                &version,
                commit_oid,
                &found_tag_names,
                &catch_up_tag,
            )
            .unwrap_or_else(|error| {
                eprintln!("Error during creating the release tag:\n\t{error}");
                process::exit(1);
            })
        });

        if verbosity >= 1 {
            if let Some(tag_name) = &release_tag {
                println!("Created tag: {tag_name}");
            }
        }

        let last_run_state = crate::undo_state::LastRunState::new(
            &cargo_toml_version,
            &version,
            commit_oid,
            catch_up_tag.clone(),
            release_tag,
        );
        let git_dir = git_repo.path();
        crate::undo_state::write(git_dir, &last_run_state).unwrap_or_else(|error| {
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

/// Compute the next semantic version for `repository`, given `config`.
///
/// Combines a baseline version with a [`SemanticVersionAction`]. `forced_action`, when given,
/// is used as-is instead of deriving one from the commits since the latest tag, in which case
/// the repository's commits aren't parsed at all.
///
/// The baseline is normally [`crate::current_version`] (the latest tag). But if
/// `cargo_toml_version` (the version currently declared in `Cargo.toml`) is *ahead* of that,
/// the tag history is missing a tag for it, e.g. it was bumped by hand without tagging. In that
/// case a catch-up tag for `cargo_toml_version` is created at `HEAD` (skipped when `noop` is
/// set, though the returned version is computed the same way either way), and it becomes the
/// baseline instead.
///
/// ## Returns
///
/// The next [`Version`], and the name of the catch-up tag if one was created (`None` if `noop`
/// was set or no reconciliation was needed).
pub fn next_version(
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

    let tag_based_current_version = crate::current_version(repository, config)?;

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
mod next_version_tests {
    use crate::command::version::next_version;
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
