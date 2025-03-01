extern crate cargo_semantic_release;
use cargo_semantic_release::Changes;
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
    // If the clap parser finds the --version or --help argument it will
    // show the version and help information respectively. Then it will exit.
    // When no arguments are found the application will just continue after
    // the parse step.
    let _ = CargoCli::parse();

    let path = env::current_dir().unwrap_or_else(|error| {
        eprintln!("Error during getting the current directory:\n\t{error}");
        process::exit(1);
    });
    println!("Current directory: {}", path.display());

    let git_repo = Repository::open(path).unwrap_or_else(|error| {
        eprintln!("Error during opening repository:\n\t{error}");
        process::exit(1);
    });

    let changes = Changes::from_repo(&git_repo).unwrap_or_else(|error| {
        eprintln!("Error during fetching changes from repository:\n\t{error}");
        process::exit(1);
    });
    println!("Changes in the repository:\n{changes}");

    let action = changes.define_action_for_semantic_version();
    println!("Action for semantic version ➡️ {action}");
}
