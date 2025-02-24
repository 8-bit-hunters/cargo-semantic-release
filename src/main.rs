extern crate cargo_semantic_release;
use cargo_semantic_release::{evaluate_changes, get_commits, Changes};
use clap::Parser;
use clap_cargo::style;
use git2::Repository;
use std::{env, process};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
enum CargoCli {
    SemanticRelease(SemanticReleaseArgs),
}

#[derive(clap::Args)]
#[command(version, about, display_name = "semantic-release")]
struct SemanticReleaseArgs {}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(style::HEADER)
    .usage(style::USAGE)
    .literal(style::LITERAL)
    .placeholder(style::PLACEHOLDER)
    .error(style::ERROR)
    .valid(style::VALID)
    .invalid(style::INVALID);

fn main() {
    let _ = CargoCli::parse();

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
