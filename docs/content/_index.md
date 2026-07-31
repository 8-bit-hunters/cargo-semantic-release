---
title: Cargo Semantic Release
---

[![pre-commit.ci status](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/pre_commit_checks.yml/badge.svg)](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/pre_commit_checks.yml)
[![cargo test status](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/tests.yml/badge.svg)](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/tests.yml)
[![Gitmoji](https://img.shields.io/badge/gitmoji-%20😜%20😍-FFDD67.svg?style=flat-square)](https://gitmoji.dev)

This project aims to create a Cargo plugin that creates semantic releases for Rust projects.

## Goals

- It can be installed as a Cargo plugin
- Works with Gitmoji commit messages
- Follows Semantic Versioning guidelines

## Installation

{{< callout type="warning" >}}
If you don't have, install the Rust toolchain.
{{< /callout >}}


Install the tool with `Cargo` with the following command.

```shell
cargo install cargo-semantic-release
```

This will globally install the `cargo-semantic-release` binary.

## Usage

You can run the tool in the directory of your choice via `Cargo` with the following command.

```shell
cargo semantic-release version
```

By default, this writes the computed next version to `Cargo.toml`, commits the bump, tags the
release, and pushes both to `origin`, printing the resulting version, e.g. `Next version: 1.2.3`.

Pass `-v` to also print the `Cargo.toml` version, the tags found, and the latest tagged version;
pass `-vv` to additionally print the commits since the last version tag, grouped into `major`,
`minor`, `patch`, and `other`.

Other flags for `version`:

- `--noop` — preview the run without writing, committing, tagging, or pushing anything.
- `--major` / `--minor` / `--patch` — force that bump instead of deriving one from commit history.
- `--no-commit` — skip creating the bump commit.
- `--no-push` — skip pushing the bump commit and any created tags to `origin`.
- `--print-tag` — print the next version's tag (e.g. `v1.2.3`) instead of the bare version.

Example output, with `-vv` to also show the commits behind the decision:
```shell
cargo semantic-release version -vv --noop
```
```
Running in no-operation mode (--noop): no files will be written, and no commits, tags, or pushes will be made.
Cargo.toml version: 1.0.0
Found tags: v1.0.0
Latest tag version: 1.0.0
Commits since the last version tag:
major:

minor:
        :sparkles: Add a new feature (a1b2c3d)

patch:
        :bug: Fix a bug (e4f5a6b)

other:
        :memo: Update documentation (c7d8e9f)

Next version: 1.1.0
```

### Undoing a run

```shell
cargo semantic-release undo
```

Restores `Cargo.toml` to its previous version and removes the bump commit and any tags `version`
created, deleting those tags from `origin` too if they were pushed. It refuses to undo if `HEAD`
has moved since the bump commit was created; pass `--force` to undo anyway.
