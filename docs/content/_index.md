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
cargo semantic-release
```

This will print out the `major`, `minor`, `patch` related changes and the `other` changes.
Also, it will indicate the recommended action for the semantic version.

Example output:
```
Current directory: /home/RustroverProjects/cargo-semantic-release
Changes in the repository:
major:

minor:
        :sparkles: Evaluate the changes to define the action for semantic version
        :sparkles: Sort committed changes according to their impact
        :children_crossing: Show CI/CD badges in the README
        :sparkles: Create function to fetch commits from a repository
        :sparkles: Print out the commits in the directory where the program was called
        :heavy_plus_sign: Add git2 as dependency
        :sparkles: Print the path of the directory where the program was called

patch:
        :wrench: Add spellcheck to pre-commit checks
        :wrench: Add clippy to pre-commit checks
        :recycle: Rename tests
        :adhesive_bandage: Fix warning for having both 'license' and 'license-file'
        :recycle: Use custom commit type
        :arrow_up: Pre-commit dependency update
        :recycle: Rename CI workflow names
        :construction_worker: Run pre-commit checks on every push
        :construction_worker: Build and test the code on every push
        :wrench: Add package information to Cargo.toml
        :pushpin: Make Cargo.lock
        :rotating_light: Pre-commit check fixes
        :technologist: Add pre-commit checks

other:
        :memo: Add documentation to the library
        :memo: Add README file
        :tada: Kick off cargo-semantic-release Rust project
        :see_no_evil: don't ignore Cargo.lock and ignore .idea folder

Action for semantic version ➡️ increment minor version
```
