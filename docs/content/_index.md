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
