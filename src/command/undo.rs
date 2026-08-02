use crate::undo_state;
use crate::version;
use cargo_semantic_release::RepositoryExtension;
use git2::{Oid, Repository};
use semver::Version;
use std::path::Path;
use std::{env, process};

#[derive(clap::Args)]
pub struct UndoArgs {
    /// Undo even if HEAD has moved since the version-bump commit was created
    #[arg(long)]
    force: bool,
}

pub fn run_undo_command(args: UndoArgs, noop: bool) {
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
        if let Some(tag_name) = &state.release_tag {
            println!("Would remove the release tag {tag_name}.");
        }
        if state.pushed {
            println!("Would remove the pushed tags from origin.");
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
    if let Some(tag_name) = &state.release_tag {
        println!("Removed the release tag {tag_name}.");
    }
    if state.pushed {
        println!("Removed the pushed tags from origin.");
    }
}

/// Reverse the repository-facing effects recorded in `state`: restore `cargo_toml_path`'s
/// declared version and, if a bump commit, catch-up tag, or release tag were created, undo
/// those too.
///
/// Refuses (returning `Err` without changing anything) if `state` recorded a bump commit and
/// `HEAD` no longer points at it, unless `force` is set.
pub fn perform_undo(
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

    if let Some(tag_name) = &state.release_tag {
        repository
            .delete_tag(tag_name)
            .map_err(|error| error.to_string())?;
    }

    if state.pushed {
        let repo_path = cargo_toml_path.parent().ok_or_else(|| {
            format!("failed to determine the repository directory from {cargo_toml_path:?}")
        })?;
        if let Some(tag_name) = &state.catch_up_tag {
            delete_remote_tag("origin", tag_name, repo_path)?;
        }
        if let Some(tag_name) = &state.release_tag {
            delete_remote_tag("origin", tag_name, repo_path)?;
        }
    }

    Ok(())
}

/// Delete the tag named `tag_name` on `remote_name`, shelling out to `git push --delete` (run
/// in `repo_path`) so the caller's existing credential helpers / SSH agent are reused as-is.
fn delete_remote_tag(remote_name: &str, tag_name: &str, repo_path: &Path) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .current_dir(repo_path)
        .arg("push")
        .arg(remote_name)
        .arg("--delete")
        .arg(tag_name)
        .status()
        .map_err(|error| format!("failed to run `git push --delete`: {error}"))?;
    if !status.success() {
        return Err(format!(
            "`git push --delete {tag_name}` exited with {status}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod perform_undo_tests {
    use crate::command::undo::perform_undo;
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
            None,
            false,
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
    fn removes_the_release_tag() {
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
        repository.create_tag("v1.1.0", bump_commit_oid).unwrap();

        let state = LastRunState::new(
            &Version::new(1, 0, 0),
            &Version::new(1, 1, 0),
            Some(bump_commit_oid),
            None,
            Some("v1.1.0".to_string()),
            false,
        );

        // When
        let result = perform_undo(&repository, &cargo_toml_path, &state, false);

        // Then
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(repository.head_commit_oid().unwrap(), pre_bump_head);
        assert!(repository
            .get_all_version_tags("v{version}")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn also_deletes_the_tags_from_origin_when_pushed() {
        // Given
        let (temp_dir, repository) = repo_init(Some(vec!["initial commit"]));
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let pre_bump_head = repository.head_commit_oid().unwrap();

        let remote_dir = tempfile::TempDir::new().unwrap();
        git2::Repository::init_bare(remote_dir.path()).unwrap();
        repository
            .remote("origin", remote_dir.path().to_str().unwrap())
            .unwrap();

        std::fs::write(
            &cargo_toml_path,
            "[package]\nname = \"foo\"\nversion = \"1.1.0\"\n",
        )
        .unwrap();
        let bump_commit_oid = repository
            .commit_file(Path::new("Cargo.toml"), "bump")
            .unwrap();
        repository.create_tag("v1.1.0", bump_commit_oid).unwrap();

        let push_status = std::process::Command::new("git")
            .current_dir(temp_dir.path())
            .args(["push", "origin", "main", "v1.1.0"])
            .status()
            .unwrap();
        assert!(push_status.success(), "test setup: seeding origin failed");

        let state = LastRunState::new(
            &Version::new(1, 0, 0),
            &Version::new(1, 1, 0),
            Some(bump_commit_oid),
            None,
            Some("v1.1.0".to_string()),
            true,
        );

        // When
        let result = perform_undo(&repository, &cargo_toml_path, &state, false);

        // Then
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(repository.head_commit_oid().unwrap(), pre_bump_head);
        let remote_repo = git2::Repository::open_bare(remote_dir.path()).unwrap();
        assert!(
            remote_repo.find_reference("refs/tags/v1.1.0").is_err(),
            "the release tag should have been deleted from origin too"
        );
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

        let state = LastRunState::new(
            &Version::new(1, 0, 0),
            &Version::new(1, 1, 0),
            None,
            None,
            None,
            false,
        );

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
            None,
            false,
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
            None,
            false,
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
