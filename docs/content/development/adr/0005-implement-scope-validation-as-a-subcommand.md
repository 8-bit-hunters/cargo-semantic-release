# Implement scope validation as a subcommand

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-15

Technical Story: Support multi-crate Cargo workspaces in cargo-semantic-release

## Context and Problem Statement

[ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) proposes using commit message scopes to explicitly
declare which packages a commit affects in a Cargo workspace. This approach, which follows conventions like Conventional
Commits and Gitmoji, requires that scope values match actual package names from the workspace `Cargo.toml`.

The problem: how and where do we validate that commit message scopes are valid (i.e., correspond to real packages in the
workspace)?

We need a mechanism to:

- Parse commit messages (regardless of convention: Gitmoji, Conventional Commits, etc.) and extract scopes
- Compare scopes against known package names from the workspace
- Provide feedback (warning or error) when scopes are invalid
- Integrate seamlessly with developer workflows (ideally at commit time)

## Decision Drivers

* Early feedback: Catch invalid scopes as early as possible in the development workflow
* Reusability: The validation logic should be usable in multiple contexts (git hooks, CI, manual checks)
* Simplicity: Minimal code complexity and maintenance burden
* Coherence: Keep related functionality together where it makes sense
* Extensibility: Allow for future enhancements (e.g., custom scope validation rules)

## Considered Options

* Inline validation in the main release logic
* Separate standalone CLI tool
* Subcommand of cargo-semantic-release
* Git hook script (shell-based)

## Decision Outcome

Chosen option: **Subcommand of cargo-semantic-release**, because it balances reusability with coherence, keeps the
implementation simple, and allows for easy integration with git hooks while sharing workspace-parsing logic with the
main codebase.

The subcommand will be: `cargo semantic-release validate-commit [--hook <commit-msg-file>]`

### Positive Consequences

* Shared code: Reuses existing workspace package discovery logic from cargo-semantic-release
* Single binary: No additional tooling to install or maintain
* Natural integration: Can be easily added to git hooks: `cargo semantic-release validate-commit --hook "$1"`
* Testable: Validation logic is isolated and easy to unit test
* Extensible: Can later be extracted into a separate crate if demand arises

### Negative Consequences

* Tighter coupling: Validation is coupled to cargo-semantic-release's release cycle
* Slightly larger binary: Adds a small amount of code to the main binary (estimated ~100-150 lines)

## Pros and Cons of the Options

### Inline validation in the main release logic

Validate scopes only during the release process, as part of version calculation.

* Good, because simplest implementation (no new entry points)
* Good, because no additional API surface
* Bad, because errors caught late (at release time, not commit time)
* Bad, because cannot be reused for git hooks or CI checks
* Bad, because mixes concerns (validation vs. version calculation)

### Separate standalone CLI tool

Create a dedicated binary like `scope-validator`.

* Good, because maximum reusability (can be used independently)
* Good, because clear separation of concerns
* Bad, because duplicates workspace-parsing logic
* Bad, because additional maintenance burden (separate crate, publishing, versioning)
* Bad, because users need to install another tool

### Subcommand of cargo-semantic-release

Add `cargo semantic-release validate-commit` as a new subcommand.

* Good, because shares workspace-parsing logic with main codebase
* Good, because single binary, no additional installation
* Good, because natural fit for git hooks
* Good, because can be extracted later if needed (refactoring, not upfront cost)
* Bad, because adds complexity to the main binary
* Bad, because coupled to cargo-semantic-release release cycle

### Git hook script (shell-based)

Provide a shell script that users can drop into `.git/hooks/commit-msg`.

* Good, because simple for users to adopt
* Bad, because shell scripts are fragile and platform-dependent
* Bad, because duplicates logic that already exists or will exist in Rust
* Bad, because hard to test and maintain

## Links

* Refines [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) (Scope-based workspace package
  filtering)
