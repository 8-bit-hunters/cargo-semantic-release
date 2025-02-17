extern crate cargo_semantic_release;
use cargo_semantic_release::{evaluate_changes, get_commits, Changes};
use git2::Repository;
use std::{env, process};

fn main() {
    let path = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {}", path.display());

    let git_repo = Repository::open(path).expect("Failed to open git repo");

    let commits = get_commits(&git_repo).unwrap_or_else(|error| {
        eprintln!("Application error: {}", error);
        process::exit(1);
    });

    let changes = Changes::sort_commits(commits);
    println!("Changes in the repository:\n{}", changes);

    let action = evaluate_changes(changes);
    println!("Action for semantic version ➡️ {}", action);
}
