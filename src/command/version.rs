use crate::VersionArgs;
use cargo_semantic_release::{
    render_tag, Changes, RepositoryExtension, SemanticReleaseConfig, SemanticVersionAction,
};
use git2::{Oid, Repository};
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

    let (version, catch_up_tag, action) = next_version(
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

        // A `Keep` action means there's nothing new to release: skip the bump commit, release
        // tag, and undo-state write, so a run with no bump-worthy commits doesn't create an
        // empty commit or clobber the state left by the last real bump.
        let mut commit_oid: Option<Oid> = None;
        let mut release_tag = None;

        if action != SemanticVersionAction::Keep {
            commit_oid = if !args.no_commit {
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

            release_tag = commit_oid.and_then(|commit_oid| {
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
        }

        let mut pushed = false;
        if !args.no_push {
            let branch_name = git_repo.head().ok().and_then(|head| {
                if head.is_branch() {
                    head.shorthand().map(str::to_string)
                } else {
                    None
                }
            });
            if let Some(refspecs) = push_refspecs(
                branch_name.as_deref(),
                commit_oid.is_some(),
                &catch_up_tag,
                &release_tag,
            ) {
                push_to_remote("origin", &refspecs, &path).unwrap_or_else(|error| {
                    eprintln!("Error during pushing to origin:\n\t{error}");
                    process::exit(1);
                });
                pushed = true;
                if verbosity >= 1 {
                    println!("Pushed {} to origin.", refspecs.join(", "));
                }
            }
        }

        if action != SemanticVersionAction::Keep {
            let last_run_state = crate::undo_state::LastRunState::new(
                &cargo_toml_version,
                &version,
                commit_oid,
                catch_up_tag.clone(),
                release_tag.clone(),
                pushed,
            );
            let git_dir = git_repo.path();
            crate::undo_state::write(git_dir, &last_run_state).unwrap_or_else(|error| {
                eprintln!("Error during recording undo state:\n\t{error}");
                process::exit(1);
            });
        }
    }

    if args.print_tag {
        println!("Next version: {}", render_tag(&config.tag_format, &version));
    } else {
        println!("Next version: {version}");
    }
}

/// The git refs (branch name and/or tag names) a `version` run should push, given what it
/// changed.
///
/// `branch_name` is `None` when `HEAD` isn't attached to a branch (e.g. a detached-HEAD CI
/// checkout), in which case there's no meaningful destination to push commits to, so the
/// branch is omitted even if `bumped` is set; tags are pushed either way.
///
/// ## Returns
///
/// `None` if there's nothing to push (no commit was made and no tag was created).
fn push_refspecs(
    branch_name: Option<&str>,
    bumped: bool,
    catch_up_tag: &Option<String>,
    release_tag: &Option<String>,
) -> Option<Vec<String>> {
    let mut refspecs = Vec::new();
    if bumped {
        if let Some(branch_name) = branch_name {
            refspecs.push(branch_name.to_string());
        }
    }
    if let Some(tag) = catch_up_tag {
        refspecs.push(tag.clone());
    }
    if let Some(tag) = release_tag {
        refspecs.push(tag.clone());
    }
    if refspecs.is_empty() {
        None
    } else {
        Some(refspecs)
    }
}

/// Push `refspecs` to `remote_name`, shelling out to `git push` (run in `repo_path`) so the
/// caller's existing credential helpers / SSH agent are reused as-is, rather than
/// re-implementing git authentication.
fn push_to_remote(remote_name: &str, refspecs: &[String], repo_path: &Path) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .current_dir(repo_path)
        .arg("push")
        .arg(remote_name)
        .args(refspecs)
        .status()
        .map_err(|error| format!("failed to run `git push`: {error}"))?;
    if !status.success() {
        return Err(format!("`git push` exited with {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod push_refspecs_tests {
    use crate::command::version::push_refspecs;

    #[test]
    fn returns_none_when_nothing_changed() {
        assert_eq!(push_refspecs(Some("main"), false, &None, &None), None);
    }

    #[test]
    fn includes_the_branch_when_bumped() {
        assert_eq!(
            push_refspecs(Some("main"), true, &None, &None),
            Some(vec!["main".to_string()])
        );
    }

    #[test]
    fn includes_the_catch_up_tag() {
        assert_eq!(
            push_refspecs(Some("main"), false, &Some("v1.0.0".to_string()), &None),
            Some(vec!["v1.0.0".to_string()])
        );
    }

    #[test]
    fn includes_the_release_tag() {
        assert_eq!(
            push_refspecs(Some("main"), false, &None, &Some("v1.1.0".to_string())),
            Some(vec!["v1.1.0".to_string()])
        );
    }

    #[test]
    fn includes_everything_when_all_present() {
        assert_eq!(
            push_refspecs(
                Some("main"),
                true,
                &Some("v1.0.0".to_string()),
                &Some("v1.1.0".to_string())
            ),
            Some(vec![
                "main".to_string(),
                "v1.0.0".to_string(),
                "v1.1.0".to_string()
            ])
        );
    }

    #[test]
    fn omits_the_branch_but_still_pushes_tags_when_head_is_detached() {
        // A detached HEAD (e.g. a CI checkout of a specific commit) has no branch name to push
        // to, but the tags created this run are still pushable by name.
        assert_eq!(
            push_refspecs(
                None,
                true,
                &Some("v1.0.0".to_string()),
                &Some("v1.1.0".to_string())
            ),
            Some(vec!["v1.0.0".to_string(), "v1.1.0".to_string()])
        );
    }

    #[test]
    fn returns_none_when_bumped_but_head_is_detached_and_no_tags_were_created() {
        assert_eq!(push_refspecs(None, true, &None, &None), None);
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
/// The next [`Version`], the name of the catch-up tag if one was created (`None` if `noop` was
/// set or no reconciliation was needed), and the [`SemanticVersionAction`] that was applied.
pub fn next_version(
    repository: &impl RepositoryExtension,
    config: &SemanticReleaseConfig,
    cargo_toml_version: &Version,
    forced_action: Option<SemanticVersionAction>,
    noop: bool,
) -> Result<(Version, Option<String>, SemanticVersionAction), Box<dyn std::error::Error>> {
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

    let version = action.apply(&baseline);
    Ok((version, catch_up_tag, action))
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
        assert_eq!(
            result.unwrap(),
            (
                Version::new(1, 0, 0),
                None,
                SemanticVersionAction::IncrementMajor
            )
        );
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
        assert_eq!(
            result.unwrap(),
            (
                Version::new(1, 2, 4),
                None,
                SemanticVersionAction::IncrementPatch
            )
        );
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
        assert_eq!(
            result.unwrap(),
            (
                Version::new(2, 0, 0),
                None,
                SemanticVersionAction::IncrementMajor
            )
        );
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
        assert_eq!(
            result.unwrap(),
            (
                Version::new(0, 0, 1),
                None,
                SemanticVersionAction::IncrementPatch
            )
        );
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
        assert_eq!(
            result.unwrap(),
            (
                Version::new(1, 0, 1),
                None,
                SemanticVersionAction::IncrementPatch
            )
        );
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
        let (version, catch_up_tag, action) = result.unwrap();
        assert_eq!(version, Version::new(2, 0, 1));
        assert_eq!(
            catch_up_tag.as_deref(),
            Some("v2.0.0"),
            "the created catch-up tag's name should have been returned"
        );
        assert_eq!(action, SemanticVersionAction::IncrementPatch);
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
    fn returns_keep_when_no_commits_warrant_a_version_change() {
        // Given
        let commit_messages = vec![":tada: initial release", ":memo: update the README"];
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
        assert_eq!(
            result.unwrap(),
            (Version::new(1, 0, 0), None, SemanticVersionAction::Keep)
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
        assert_eq!(
            result.unwrap(),
            (
                Version::new(2, 0, 1),
                None,
                SemanticVersionAction::IncrementPatch
            )
        );
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
