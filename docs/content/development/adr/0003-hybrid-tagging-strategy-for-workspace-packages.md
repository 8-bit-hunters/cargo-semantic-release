# Hybrid tagging strategy for workspace packages

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-15

Technical Story: Support multi-crate Cargo workspaces in cargo-semantic-release

## Context and Problem Statement

When working with Cargo workspaces containing multiple packages, we need a strategy for version tagging. The question
is: should each package have its own independent version/tag, should the entire workspace share a single version/tag, or
should we support both?

This decision affects how versions are calculated, how tags are created, and how releases are published. It also needs
to be flexible enough to accommodate different workflows (independent package releases vs. lockstep workspace releases).

## Decision Drivers

* Flexibility: Support both independent and lockstep release workflows
* Idiomatic Rust: Each crate typically has its own version in the Rust ecosystem
* Backward Compatibility: Existing single-crate projects should continue to work
* Explicit Control: Users should be able to configure the behavior
* Tooling Integration: Should work with Cargo's native understanding of workspaces
* Requirements: [REQ-015](../../../../plm/req/REQ-015.md) (per-package versions and tags)

## Considered Options

* Shared Version for All Packages
* Per-Package Independent Versions
* Hybrid: Per-Package Versions with Optional Workspace Version

## Decision Outcome

Chosen option: **Hybrid: Per-Package Versions with Optional Workspace Version**, because it provides maximum
flexibility, aligns with Rust/Cargo conventions (each crate has its own version), and allows users to optionally enable
workspace-level versioning when needed.

### Positive Consequences

* Idiomatic: Matches how Rust developers expect workspaces to work (each crate has its own version)
* Flexible: Users can choose independent or lockstep releases via configuration
* Granular: Each package can be released on its own cadence
* Backward Compatible: Single-crate projects continue to work unchanged
* Explicit: Users have control over whether workspace-level tagging is enabled

### Negative Consequences

* Complexity: More moving parts to manage (per-package tags + optional workspace tags)
* Tag Proliferation: Many tags in the repository if all packages release frequently
* Configuration Burden: Users need to understand and configure the strategy

### Configuration

The hybrid strategy would be controlled by a configuration option:

```toml
[workspace.metadata.semantic-release]
# Default: per-package versions only
enable_workspace_version = false  # or true

# When workspace version is enabled, define its tag format
workspace_tag_format = "{name}-v{version}"
```

When `enable_workspace_version = false` (default):

- Each package is versioned and tagged independently
- Only per-package tags are created

When `enable_workspace_version = true`:

- Each package still has its own version and tag
- Additionally, a workspace-level tag is created that represents the combination of all package versions
- The workspace version could be derived from package versions or maintained separately

## Pros and Cons of the Options

### Shared Version for All Packages

All packages in the workspace share a single version and tag (e.g., `my-workspace-v1.2.3`).

* Good, because simplest to understand and implement
* Good, because single source of truth for the workspace version
* Good, because lockstep releases are guaranteed
* Bad, because not idiomatic Rust (crates typically have independent versions)
* Bad, because cannot release packages independently
* Bad, because all packages bump version even if only one changed

### Per-Package Independent Versions

Each package maintains its own version and tag (e.g., `my-crate-v1.2.3`, `other-crate-v0.4.5`).

* Good, because idiomatic Rust (each crate has its own version)
* Good, because packages can release independently
* Good, because precise versioning per package
* Bad, because no concept of workspace-level version
* Bad, because may not suit teams that want lockstep releases

### Hybrid: Per-Package Versions with Optional Workspace Version

Each package has its own version and tag by default. Additionally, users can optionally enable a workspace-level
version/tag that aggregates all package versions.

* Good, because idiomatic Rust by default
* Good, because supports both independent and lockstep workflows
* Good, because configurable based on user needs
* Good, because backward compatible with single-crate projects
* Bad, because more complex implementation
* Bad, because users need to configure and understand the options

## Links

* Refines [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) (Gitmoji scope-based workspace package
  filtering)
