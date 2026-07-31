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

### Running in CI

`cargo semantic-release version` commits the bump and pushes it, so it needs `HEAD` checked
out on a real branch. Most CI providers check out a detached `HEAD` by default (e.g. GitHub
Actions' `actions/checkout`), which leaves nothing for the push to update — the bump commit
would only be reachable via its tag, never merged into your branch. Point the checkout at the
branch explicitly, e.g. with `actions/checkout`:

```yaml
- uses: actions/checkout@v4
  with:
    ref: ${{ github.ref_name }}
```

## Configuration

By default, `cargo-semantic-release` works out of the box with no configuration needed. You can
customize its behavior with a config table shaped like
[python-semantic-release's](https://python-semantic-release.readthedocs.io/en/latest/configuration/configuration.html#config)
`[tool.semantic_release]`, so existing PSR config using its `emoji` commit parser can be reused
as-is.

Config is discovered in this order, and the first one found wins:

1. `[package.metadata.semantic_release]` in `Cargo.toml`
2. a standalone `semantic_release.toml` file (bare table, no wrapper)

If neither is present, the tool falls back to its built-in defaults.

### Options

- `tag_format` — the shape of version tags, e.g. `"v{version}"`. The literal `{version}`
  placeholder marks where the semantic version sits.
- `commit_parser_options.major_tags` / `minor_tags` / `patch_tags` — the Gitmoji shortcodes (e.g.
  `:boom:`) or literal emoji (e.g. `💥`) that trigger each level of version bump. A commit whose
  leading Gitmoji isn't listed in any of these doesn't affect the bump decision.

### Example: `Cargo.toml`

```toml
[package.metadata.semantic_release]
tag_format = "v{version}"

[package.metadata.semantic_release.commit_parser_options]
major_tags = [":boom:"]
minor_tags = [":sparkles:", ":children_crossing:"]
patch_tags = [":bug:", ":zap:"]
```

### Example: standalone `semantic_release.toml`

```toml
tag_format = "v{version}"

[commit_parser_options]
major_tags = [":boom:"]
minor_tags = [":sparkles:", ":children_crossing:"]
patch_tags = [":bug:", ":zap:"]
```

### Defaults

```toml
tag_format = "v{version}"

[commit_parser_options]
major_tags = [":boom:"]
minor_tags = [
    ":sparkles:", ":children_crossing:", ":lipstick:", ":iphone:", ":egg:",
    ":chart_with_upwards_trend:", ":heavy_plus_sign:", ":heavy_minus_sign:", ":passport_control:",
]
patch_tags = [
    ":art:", ":ambulance:", ":lock:", ":bug:", ":zap:", ":goal_net:", ":alien:", ":wheelchair:",
    ":speech_balloon:", ":mag:", ":fire:", ":white_check_mark:", ":closed_lock_with_key:",
    ":rotating_light:", ":green_heart:", ":arrow_down:", ":arrow_up:", ":pushpin:",
    ":construction_worker:", ":recycle:", ":wrench:", ":hammer:", ":globe_with_meridians:",
    ":package:", ":truck:", ":bento:", ":card_file_box:", ":loud_sound:", ":mute:",
    ":building_construction:", ":camera_flash:", ":label:", ":seedling:",
    ":triangular_flag_on_post:", ":dizzy:", ":adhesive_bandage:", ":monocle_face:", ":necktie:",
    ":stethoscope:", ":technologist:", ":thread:", ":safety_vest:",
]
```

## Library

The utility functions for the binary are available in a [library crate](https://docs.rs/crate/cargo-semantic-release/).

## Links

- [Homepage](https://8-bit-hunters.github.io/cargo-semantic-release/)
- [Crates.io](https://crates.io/crates/cargo-semantic-release)
- [GitHub](https://github.com/8-bit-hunters/cargo-semantic-release)
