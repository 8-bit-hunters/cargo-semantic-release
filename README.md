# Cargo Semantic Release

[![pre-commit.ci status](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/pre_commit_checks.yml/badge.svg)](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/pre_commit_checks.yml)
[![cargo test status](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/tests.yml/badge.svg)](https://github.com/8-bit-hunters/cargo-semantic-release/actions/workflows/tests.yml)
[![Gitmoji](https://img.shields.io/badge/gitmoji-%20😜%20😍-FFDD67.svg?style=flat-square)](https://gitmoji.dev)

This project aims to create a Cargo plugin that creates semantic releases for Rust projects.

## Goals

- It can be installed as a Cargo plugin
- Works with Gitmoji commit messages
- Follows Semantic Versioning guidelines

## Installation

If you don't have, install the Rust toolchain.

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

## Library

The utility functions for the binary are available in a [library crate](https://docs.rs/crate/cargo-semantic-release/).

## Links

- [Homepage](https://8-bit-hunters.github.io/cargo-semantic-release/)
- [Crates.io](https://crates.io/crates/cargo-semantic-release)
- [GitHub](https://github.com/8-bit-hunters/cargo-semantic-release)
