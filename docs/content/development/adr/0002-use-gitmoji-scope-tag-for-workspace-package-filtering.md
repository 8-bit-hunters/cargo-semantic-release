# Use Gitmoji Scope Tag for Workspace Package Filtering

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-15

Technical Story: Support multi-crate Cargo workspaces in cargo-semantic-release

## Context and Problem Statement

cargo-semantic-release is currently designed for single-crate Rust projects, where one `Cargo.toml` file defines a
single package version. However, Cargo workspaces allow multiple packages (crates) to coexist in a single repository,
each with their own version in their respective `Cargo.toml` files.

The challenge is: how do we determine which commits affect which packages in a workspace?

In a monorepo/workspace scenario:

- A commit might modify files in only one package
- A commit might modify files in multiple packages
- A commit might only modify workspace-level files (root `Cargo.toml`, README, CI config, etc.)
- Each package should have its own semantic version calculated based on the commits that affect it

Without a mechanism to associate commits with packages, we would need to parse git diffs for every commit and map file
paths to package directories — an approach that is complex, slow, fragile, and implicit.

## Decision Drivers

* Simplicity: Prefer simple, maintainable code over complex git operations
* Performance: Avoid expensive git diff parsing when possible
* Explicitness: Prefer developer-declared intent over inferred behavior
* Conventional: Follow existing standards where possible (Conventional Commits)
* Backward Compatibility: Ensure existing projects continue to work
* Developer Experience: Make the behavior intuitive and controllable by developers
* Requirements: [REQ-012](../../../../plm/req/REQ-012.md) (the commit scope declares the affected packages) and [REQ-013](../../../../plm/req/REQ-013.md)
  (workspace package discovery)

## Considered Options

* File-Path Based Filtering
* Scope-Based Filtering (Using Gitmoji Commit Message Scope)
* Hybrid Approach (Scope + File-Path Fallback)
* Shared Version for All Packages

## Decision Outcome

Chosen option: "Scope-Based Filtering", because it provides the best balance of simplicity, explicitness, performance,
and convention adherence. It requires minimal code changes (~100-150 lines) while being fast (string parsing only) and
explicit (developers declare which packages are affected).

### Positive Consequences

* Simple Implementation: less complexity than file-path filtering
* Fast Performance: String parsing only, no git diff operations
* Explicit Intent: Developers control which packages are affected
* Conventional: Extends Gitmoji naturally, similar to Conventional Commits pattern
* Backward Compatible: Works with existing commits (treated as workspace-level)
* Flexible: Supports single, multiple, or no scopes
* Maintainable: Easy to understand and extend
* Testable: Simple string parsing is easy to unit test

### Negative Consequences

* Developer Discipline Required: Developers must remember to add scopes for package-specific commits
* Misattribution Risk: Without scopes, commits may be incorrectly attributed (mitigated by default behavior and
  warnings)
* Scope Validation Needed: Should warn about unknown scopes
* Learning Curve: New convention to learn (though minimal)

### Version Calculation Strategy

Each package's semantic version is calculated **independently** based only on commits that explicitly list that package
in their Gitmoji scope.

Unscoped commits are **not** applied to any package version. They are treated as workspace-level changes that do not
trigger package version bumps. This encourages developers to be explicit about package associations.

#### How it Works

For each package in the workspace:

1. Collect all commits that have the package name in their scope (e.g., `:sparkles: Add feature (my-crate)`)
2. Apply the semantic version bump logic based on those commits only
3. The package's current version (from its `Cargo.toml`) is the starting point

#### Examples

| Commit Message                         | Affects `my-crate` | Affects `other-crate` |
|----------------------------------------|-------------------:|----------------------:|
| `:sparkles: Add feature (my-crate)`    |              ✅ Yes |                  ❌ No |
| `:bug: Fix bug (my-crate,other-crate)` |              ✅ Yes |                 ✅ Yes |
| `:memo: Update README` (no scope)      |               ❌ No |                  ❌ No |
| `:wrench: Update CI config` (no scope) |               ❌ No |                  ❌ No |

## Pros and Cons of the Options

### File-Path Based Filtering

Analyze each commit's file changes and map them to package directories.

* Good, because no developer action required (automatic)
* Good, because based on actual file changes, not developer intent (accurate)
* Good, because can handle file-level changes precisely (fine-grained)
* Bad, because requires significant git diff parsing logic (complex implementation)
* Bad, because each package analysis requires traversing and parsing commit diffs (performance overhead)
* Bad, because edge cases with file renames, deletions, and symlinks (fragile)
* Bad, because commit-to-package association is inferred, not declared (implicit behavior)

### Scope-Based Filtering (Using Gitmoji Commit Message Scope)

Use the scope portion of Gitmoji commit messages to explicitly declare which packages a commit affects.

* Good, because only requires string parsing, no git diff operations (simple)
* Good, because developers declare which packages are affected (explicit)
* Good, because minimal computational overhead (fast)
* Good, because follows Conventional Commits pattern (`type(scope): message`) (conventional)
* Good, because supports single package, multiple packages, or workspace-level (flexible)
* Good, because commits without scopes work (backward compatible)
* Bad, because requires developer discipline to add scopes
* Bad, because commits without scopes may be misattributed if default is wrong
* Bad, because scope validation needed (ensure scopes match actual package names)

### Hybrid Approach (Scope + File-Path Fallback)

Use scope-based filtering as the primary method, with file-path filtering as a fallback for commits without scopes.

* Good, because best of both worlds: explicit when declared, automatic otherwise
* Good, because maximum backward compatibility
* Good, because handles both scoped and unscoped commits
* Bad, because more complex implementation
* Bad, because still requires file-path parsing logic
* Bad, because inconsistent behavior (sometimes explicit, sometimes inferred)

### Shared Version for All Packages

Treat the entire workspace as a single unit with one shared version.

* Good, because simplest implementation (no changes needed to core logic)
* Good, because matches some workflows (lockstep releases)
* Bad, because not idiomatic Rust (each package typically has independent version)
* Bad, because can't release packages independently
* Bad, because all packages bump version even if only one changed

## Links

* Refines [Conventional Commits](https://www.conventionalcommits.org/) — Inspires the scope format
* Refines [Gitmoji](https://gitmoji.dev/) — Existing commit message convention
* Refined by [ADR-0005](0005-implement-scope-validation-as-a-subcommand.md) (Scope validation subcommand)
* Refined by [ADR-0003](0003-hybrid-tagging-strategy-for-workspace-packages.md) (Hybrid tagging strategy)
