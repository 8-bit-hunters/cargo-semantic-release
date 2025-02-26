extern crate cargo_semantic_release;
use cargo_semantic_release::Changes;
use clap::Parser;
use clap_cargo::style;
use git2::Repository;
use std::env;

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

    let changes = Changes::from_repo(&git_repo);
    println!("Changes in the repository:\n{changes}");

    let action = changes.define_action_for_semantic_version();
    println!("Action for semantic version ➡️ {action}");
}
