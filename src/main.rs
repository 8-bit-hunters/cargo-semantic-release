extern crate cargo_semantic_release;
use cargo_semantic_release::Changes;
use git2::Repository;
use std::env;

fn main() {
    let path = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {}", path.display());

    let git_repo = Repository::open(path).expect("Failed to open git repo");

    let changes = Changes::from_repo(&git_repo);
    println!("Changes in the repository:\n{changes}");

    let action = changes.define_action_for_semantic_version();
    println!("Action for semantic version ➡️ {action}");
}
