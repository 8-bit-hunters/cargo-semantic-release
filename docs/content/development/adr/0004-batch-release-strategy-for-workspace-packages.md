# Batch release strategy for workspace packages

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-15

Technical Story: Support multi-crate Cargo workspaces in cargo-semantic-release

## Context and Problem Statement

With a Cargo workspace containing multiple packages that may depend on each other, we need a strategy for releasing and
publishing versions. The question is: should packages be released independently on their own cadence, or should we
coordinate releases across the workspace?

This decision is influenced by:

- [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) which defines how commits are associated with
  packages
- [ADR-0003](0003-hybrid-tagging-strategy-for-workspace-packages.md) which defines the tagging strategy
- The fact that intra-workspace dependencies use `path = "../..."` which cannot be published to crates.io

## Decision Drivers

* Consistency: Ensure published packages have compatible versions
* Atomicity: Treat the workspace release as a single unit of change
* Simplicity: Minimal complexity for developers to understand and use
* Compatibility: Work with Cargo's native behavior for path dependencies
* Traceability: Single commit and tags make it easy to track what changed

## Considered Options

* Independent Package Releases
* Batch Workspace Release
* Topological (Dependency-Ordered) Independent Releases

## Decision Outcome

Chosen option: **Batch Workspace Release**, because it ensures version consistency across all published packages, aligns
with the hybrid tagging strategy, and works with Cargo's requirement that path dependencies must be converted to version
dependencies for publishing.

### Positive Consequences

* Atomic versioning: All packages move to new versions together in a single commit
* Version consistency: External consumers get all packages with matching, compatible versions
* Clean git history: Single commit with all version changes, easy to revert if needed
* Lock file consistency: `Cargo.lock`/`uv.lock` reflects the exact versions being published
* Works with path deps: Using `version + path` syntax satisfies both local development and publishing requirements

### Negative Consequences

* Less flexibility: Packages cannot release independently on their own cadence
* Must publish in order: Dependencies must be published before dependents (though the version calculation is atomic)

### The Batch Release Flow

#### 1. Calculate Versions

For each package in the workspace, calculate the new semantic version based on commits that have that package in their
scope (per [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md)).

#### 2. Update Package Manifests

For each package:

- Update the `version` field in its `Cargo.toml` to the new calculated version

#### 3. Update Intra-Workspace Dependencies

For dependencies on other workspace packages, use the **version + path** syntax:

```toml
[dependencies]
other-crate = { version = "0.5.0", path = "../other-crate" }
```

This allows:

- Local development: Cargo uses the `path` for fast, local builds
- Publishing: Cargo uses the `version` for crates.io dependencies

#### 4. Update Lock File

Regenerate `Cargo.lock` (or `uv.lock`) to reflect all the new versions.

#### 5. Commit Version Changes

Create a single commit with all version changes:

```
:bookmark: Bump versions: my-crate@1.2.0, other-crate@0.5.0, workspace@1.2.0
```

#### 6. Create Tags

Create Git tags for each package (and workspace if enabled), all pointing to the same commit:

- `my-crate-v1.2.0`
- `other-crate-v0.5.0`
- `workspace-v1.2.0` (if hybrid tagging is enabled)

#### 7. Publish to crates.io

Run `cargo publish` in dependency order (leaf packages first, then packages that depend on them). The `version + path`
syntax ensures that publishing works correctly even though local development uses path dependencies.

### Handling Path Dependencies

**Problem:** Cargo cannot publish packages with pure `path = "../..."` dependencies to crates.io, because crates.io has
no concept of filesystem paths.

**Solution:** Use the `version + path` syntax for all intra-workspace dependencies:

```toml
[dependencies]
my-lib = { version = "0.1.0", path = "../my-lib" }
```

This is the idiomatic Cargo pattern that:

- Uses the path for fast local development and testing
- Uses the version when publishing to crates.io
- Allows the package to be consumed by external users via crates.io

## Pros and Cons of the Options

### Independent Package Releases

Each package publishes independently as soon as its version changes.

* Good, because maximum flexibility, packages release on their own cadence
* Good, because idiomatic Rust (crates typically release independently)
* Bad, because may publish incompatible versions (A depends on B@2.0.0 but B@2.0.0 isn't published yet)
* Bad, because complex to coordinate version compatibility
* Bad, because requires `version + path` syntax anyway, reducing the benefit

### Batch Workspace Release

All packages with version changes are committed and tagged together, then published in dependency order.

* Good, because ensures version consistency across all packages
* Good, because single atomic commit, easy to review and revert
* Good, because aligns with hybrid tagging strategy
* Good, because works with Cargo's path dependency limitations
* Bad, because less flexible (packages cannot release independently)
* Bad, because must publish in dependency order

### Topological (Dependency-Ordered) Independent Releases

Packages release independently but in dependency order (leaf packages first).

* Good, because allows independent releases
* Good, because ensures compatibility
* Bad, because complex to implement ordering logic
* Bad, because still requires coordination between package releases
* Bad, because loses atomicity benefits of single commit

## Links

* Refines [ADR-0002](0002-use-gitmoji-scope-tag-for-workspace-package-filtering.md) (Scope-based workspace package filtering
  and version calculation)
* Refines [ADR-0003](0003-hybrid-tagging-strategy-for-workspace-packages.md) (Hybrid tagging strategy)
* Related to [ADR-0005](0005-implement-scope-validation-as-a-subcommand.md) (Scope validation subcommand)
