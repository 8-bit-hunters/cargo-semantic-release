# Aggregated changelog strategy for workspace packages

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-15

Technical Story: Support multi-crate Cargo workspaces in cargo-semantic-release

## Context and Problem Statement

With a Cargo workspace containing multiple packages, each with its own version history, we need a strategy for
generating changelogs. The question is: should each package have its own changelog, should there be a single
workspace-level changelog, or should we support both?

This decision is influenced by:

- [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) which defines how commits are associated with
  packages
- [ADR-0004](0004-batch-release-strategy-for-workspace-packages.md) which defines the batch release flow
- The need to capture both package-specific changes (scoped commits) and workspace-level changes (unscoped commits)

## Decision Drivers

* Standard compliance: Follow Rust ecosystem conventions (per-package changelogs)
* Completeness: Capture all changes, including workspace-level (unscoped) commits
* Maintainability: Avoid duplication, make it easy to keep changelogs up to date
* Discoverability: Make it easy for users to find relevant changes
* Automation: Enable tooling to generate changelogs automatically

## Considered Options

* Per-Package Changelogs Only
* Single Workspace Changelog
* Both (Per-Package + Workspace)
* Per-Package + Aggregated Workspace Changelog

## Decision Outcome

Chosen option: **Per-Package + Aggregated Workspace Changelog**, because it provides standard-compliant per-package
changelogs while also offering a complete workspace-level view, and aligns with the scope-based filtering and batch
release strategies.

### Positive Consequences

* Standard compliant: Each package has its own `CHANGELOG.md` as expected by Rust users
* Complete workspace view: Root `CHANGELOG.md` provides a single source for all changes
* No duplication: Package changes are sourced from per-package changelogs
* Workspace changes captured: Unscoped commits appear in the root changelog
* Automatable: Can be generated automatically from commit history and scopes

### Negative Consequences

* More files to maintain: Per-package changelogs + workspace changelog
* Potential for drift: If per-package changelogs are manually edited, they may diverge from the aggregated workspace
  changelog

### Changelog Structure

#### Per-Package Changelogs

Each package has its own `CHANGELOG.md` in its directory:

```
workspace/
└── crates/
    ├── my-crate/
    │   ├── Cargo.toml
    │   └── CHANGELOG.md      # Only scoped commits for my-crate
    └── other-crate/
        ├── Cargo.toml
        └── CHANGELOG.md      # Only scoped commits for other-crate
```

Content is derived from commits that have that specific package in their scope.

#### Workspace (Root) Changelog

The root `CHANGELOG.md` aggregates content from all per-package changelogs **plus** workspace-level changes:

```
workspace/
├── CHANGELOG.md              # Aggregated from all package changelogs + unscoped commits
└── crates/
    └── ...
```

Structure:

```markdown
# Changelog

## [1.2.0] - 2026-09-14

### my-crate

- :sparkles: Add new feature X
- :bug: Fix edge case in parser

### other-crate

- :sparkles: Add Y functionality
- :wrench: Refactor internal API

### Workspace

- :memo: Update README
- :wrench: Update CI configuration
```

#### How Aggregation Works

1. **Collect per-package changelogs**: For each package, read its `CHANGELOG.md` and extract entries for the current
   version
2. **Collect workspace-level changes**: Extract commits that have no scope (treated as workspace-level)
3. **Merge and organize**:
    - Group changes by package name
    - Add a "Workspace" section for unscoped commits
    - Sort chronologically

### Configuration

This strategy can be controlled via workspace metadata:

```toml
[workspace.metadata.semantic-release]
# Default: generate per-package changelogs only
generate_workspace_changelog = true

# Location of workspace changelog
workspace_changelog_path = "CHANGELOG.md"

# Include workspace-level (unscoped) changes in workspace changelog
include_workspace_changes = true
```

## Pros and Cons of the Options

### Per-Package Changelogs Only

Each package has its own `CHANGELOG.md` with only its scoped commits.

* Good, because standard Rust convention
* Good, because each package's changelog reflects only its own changes
* Good, because easy to consume when package is used independently
* Bad, because no single view of all workspace changes
* Bad, because workspace-level changes (unscoped commits) are lost

### Single Workspace Changelog

One `CHANGELOG.md` at workspace root with all changes.

* Good, because single source of truth for the entire workspace
* Good, because easy to see all changes in one place
* Bad, because non-standard (users expect per-package changelogs)
* Bad, because hard to extract changelog for a single package
* Bad, because doesn't work well if packages are consumed independently

### Both (Per-Package + Workspace)

Each package has its own changelog, plus a separate workspace changelog.

* Good, because standard per-package changelogs
* Good, because workspace-level overview available
* Bad, because duplication of content between per-package and workspace changelogs
* Bad, because maintenance burden (two sets of changelogs to keep in sync)

### Per-Package + Aggregated Workspace Changelog

Each package has its own changelog, and the workspace changelog is **aggregated** from the per-package changelogs plus
workspace-level changes.

* Good, because standard per-package changelogs
* Good, because complete workspace view
* Good, because no duplication (workspace changelog is derived)
* Good, because workspace-level changes are captured
* Good, because automatable
* Bad, because more complex to implement
* Bad, because workspace changelog is derived, not hand-written

## Links

* Refines [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) (Scope-based workspace package filtering
  and version calculation)
* Refines [ADR-0004](0004-batch-release-strategy-for-workspace-packages.md) (Batch release strategy)
* Refines [ADR-0003](0003-hybrid-tagging-strategy-for-workspace-packages.md) (Hybrid tagging strategy)
* Related to [ADR-0005](0005-implement-scope-validation-as-a-subcommand.md) (Scope validation subcommand)
