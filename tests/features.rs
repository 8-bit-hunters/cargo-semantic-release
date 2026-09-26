//! Executable specification: runs the Gherkin scenarios in `tests/features` against the real
//! binary, in a throwaway git repository.
//!
//! The scenarios are written in the language of the work being published, without a word about
//! git, Cargo, or this tool's command line: everything technical belongs in the step definitions
//! below. Translating a scenario's vocabulary into a repository, a commit, and a command line is
//! this file's only job.
//!
//! Every scenario is tagged with the UID of the Doorstop item in `plm/tst` that traces it back to
//! a requirement, e.g. `@TST-001`. Keep the tag and the item in step.

use cargo_semantic_release::test_util::{repo_init, RepositoryTestExtensions};
use cucumber::{gherkin::Step, given, then, when, World};
use git2::Repository;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, str};
use tempfile::TempDir;

/// The state a scenario builds up: the throwaway project, and what the last command did.
#[derive(Debug, Default, World)]
struct CliWorld {
    /// Kept alive so the temporary directory outlives the scenario.
    project: Option<TempDir>,
    stdout: String,
    success: bool,
    /// `HEAD` and the tag names as they were just before the command ran, so a scenario can
    /// assert that nothing was published.
    head_before: Option<String>,
    tags_before: Vec<String>,
}

impl CliWorld {
    fn path(&self) -> &Path {
        self.project
            .as_ref()
            .expect("the work should have been set up by a Given step")
            .path()
    }

    fn cargo_toml(&self) -> PathBuf {
        self.path().join("Cargo.toml")
    }

    fn repo(&self) -> Repository {
        Repository::open(self.path()).expect("the work should be kept in a git repository")
    }

    fn head(&self) -> String {
        self.repo()
            .head()
            .expect("HEAD should resolve")
            .peel_to_commit()
            .expect("HEAD should point at a commit")
            .id()
            .to_string()
    }

    fn tags(&self) -> Vec<String> {
        let repo = self.repo();
        let names = repo.tag_names(None).expect("tags should be listable");
        let mut tags: Vec<String> = names.iter().flatten().map(str::to_string).collect();
        tags.sort();
        tags
    }

    /// Run the tool in the throwaway project, as Cargo would invoke it.
    fn invoke(&mut self, args: &[&str]) {
        self.head_before = Some(self.head());
        self.tags_before = self.tags();

        let output = Command::new(env!("CARGO_BIN_EXE_cargo-semantic-release"))
            .current_dir(self.path())
            .arg("semantic-release")
            .args(args)
            .output()
            .expect("the binary should be runnable");

        self.success = output.status.success();
        self.stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    }
}

/// Translate a recorded change, as a scenario describes it, into the Gitmoji commit message that
/// expresses it.
///
/// This mapping is the whole of what the scenarios do not have to know: which emoji carries which
/// meaning, and that a change is recorded as a commit at all.
fn commit_message_for(change: &str) -> String {
    let intention = match change {
        "a change that breaks how the work is used" => ":boom:",
        "an addition that leaves existing use working" => ":sparkles:",
        "a correction" => ":bug:",
        "a note about the work" => ":memo:",
        other => panic!(
            "unknown change {other:?}: add it to `commit_message_for` before using it in a scenario"
        ),
    };
    format!("{intention} {change}")
}

fn cargo_toml_contents(version: &str) -> String {
    format!("[package]\nname = \"demo\"\nversion = \"{version}\"\nedition = \"2021\"\n")
}

#[given(expr = "the work was last published as version {string}")]
fn last_published_as(world: &mut CliWorld, version: String) {
    let publication = ":tada: the work is first published";
    let (temp_dir, repository) = repo_init(Some(vec![publication]));
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        cargo_toml_contents(&version),
    )
    .expect("Cargo.toml should be writable");

    let published_commit = repository
        .find_commit_by_message(publication)
        .expect("the publication should be found");
    repository.add_tag(published_commit, &format!("v{version}"));

    world.project = Some(temp_dir);
}

#[given("the following changes were recorded since then")]
fn changes_were_recorded(world: &mut CliWorld, step: &Step) {
    let table = step.table().expect("the step should have a data table");
    let repository = world.repo();
    // The first row names the column.
    for row in table.rows.iter().skip(1) {
        let change = row.first().expect("each row should describe a change");
        repository.add_commit(&commit_message_for(change));
    }
}

#[when("a version is proposed for the next publication")]
fn a_version_is_proposed(world: &mut CliWorld) {
    world.invoke(&["version", "--noop"]);
}

#[when("a major version is demanded for the next publication")]
fn a_major_version_is_demanded(world: &mut CliWorld) {
    world.invoke(&["version", "--noop", "--major"]);
}

#[then(expr = "the proposed version is {string}")]
fn the_proposed_version_is(world: &mut CliWorld, expected: String) {
    assert!(
        world.success,
        "expected a version to be proposed, but the run failed:\n{}",
        world.stdout
    );
    let reported = format!("Next version: {expected}");
    assert!(
        world.stdout.contains(&reported),
        "expected the proposed version to be {expected:?}, but the run reported:\n{}",
        world.stdout
    );
}

#[then(expr = "the work still carries version {string}")]
fn the_work_still_carries_version(world: &mut CliWorld, expected: String) {
    let contents = fs::read_to_string(world.cargo_toml()).expect("Cargo.toml should be readable");
    assert!(
        contents.contains(&format!("version = \"{expected}\"")),
        "expected the work to still carry version {expected:?}, but it reads:\n{contents}"
    );
}

#[then("no publication is recorded")]
fn no_publication_is_recorded(world: &mut CliWorld) {
    assert_eq!(
        world.head_before.as_deref(),
        Some(world.head().as_str()),
        "nothing should have been added to the record of changes"
    );
    assert_eq!(
        world.tags_before,
        world.tags(),
        "no publication should have been marked"
    );
}

fn main() {
    futures::executor::block_on(CliWorld::run("tests/features"));
}
