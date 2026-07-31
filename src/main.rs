extern crate cargo_semantic_release;
use cargo_semantic_release::SemanticVersionAction;
use clap::Parser;
use clap_cargo::style;

mod command;
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

    /// Skip pushing the version-bump commit and any created tags to origin
    #[arg(long)]
    no_push: bool,
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
            command::version::run_version_command(version_args, verbosity, noop)
        }
        SemanticReleaseCommand::Undo(undo_args) => command::undo::run_undo_command(undo_args, noop),
    }
}
