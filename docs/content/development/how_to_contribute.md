---
title: How to Contribute
cascade:
    type: docs
---

## How to Contribute

### 🛠 Setting Up the Project

1. **Install Rust**
   Ensure you have Rust installed. We recommend using [rustup](https://rustup.rs/):

   ```shell
   rustup update
   ```

2. **Install Pre-commit Hooks**
   This project uses `pre-commit` to enforce code quality checks automatically. Install and activate the hooks:

   ```shell
   uvx pre-commit
   uvx pre-commit install
   ```

   Alternatively, you can use [`prek`](https://prek.j178.dev/), a Rust reimplementation of `pre-commit` that reads
   the same `.pre-commit-config.yaml`. See the [installation instructions](https://prek.j178.dev/installation/) for
   your platform, then run:

   ```shell
   prek install
   ```

   > **Note:** `pre-commit install` / `prek install` register a **git** hook. If you use [`jj`](https://jj-vcs.github.io/jj/)
   > instead of `git` for commits, that hook won't run automatically — jj has no equivalent hook mechanism yet. In
   > that case, run the checks manually before pushing:
   >
   > ```shell
   > prek run --all-files   # or: pre-commit run --all-files
   > ```

3. **Install Spellcheck**
   This project uses `spellcheck`. Install the tool:

    ```shell
    cargo install cargo-spellcheck
    ```

4. **Build and Test**
   Run the following commands to verify everything is working:

   ```sh
   cargo build
   cargo test
   ```

5. **Build the Docs**
   The project documentation is generated using Hugo. Install the following requirements:

   - [Go](https://go.dev/) — Hugo pulls the Hextra theme in as a Hugo module
   - [Hugo](https://gohugo.io/installation/) — the extended build

   Then serve the site through the project CLI, which publishes the generated requirement pages and
   clears Hugo's previous output first — in that order:

   ```shell
   tools/project docs serve
   ```

### 🚀 Making Contributions

- Follow Rust’s [coding conventions](https://doc.rust-lang.org/1.0.0/style/) and ensure your code is **formatted** with
  `cargo fmt`:

  ```sh
  cargo fmt --all
  ```

- Run `clippy` for linting:

  ```sh
  cargo clippy --all-targets --all-features
  ```

- Ensure all tests pass:

  ```sh
  cargo test
  ```

- Run **pre-commit checks** before committing:

  ```sh
  prek run --all-files   # or: pre-commit run --all-files
  ```

### 📋 Requirements and Behaviours

Requirements live in `plm/req/` and are managed with [Doorstop](https://doorstop.readthedocs.io/); the
behaviours that verify them are Gherkin scenarios in `tests/features/`. See
[Requirements](requirements/_index.md) for how the two fit together.

Requirements follow the SOPHIST MASTeR sentence pattern — a named system, an obligation, one main
verb, and `IF` / `AS SOON AS` / `AS LONG AS` for conditions. Scenarios are written in the language of
the work being published, not of the tool: no git, Cargo,
flags, or output, and a title that gives the precondition and the action but not the outcome.
Everything technical belongs in the step definitions in `tests/features.rs`. `TST` items are generated
from the feature files — never edit one by hand.

If you change a requirement or add a behaviour:

```shell
tools/project --help        # every task the project has
tools/project req check     # validate the tree
tools/project req sync      # regenerate the items from the feature files
tools/project req review    # accept your own changes
tools/project trace check   # scenarios, items and ADR citations
cargo test --test features            # run the scenarios
```

`project check` runs the same checks as the **Requirements** workflow, in the same order.

Both checks also run in CI, which is the gate that counts — the Doorstop checks are deliberately
not pre-commit hooks, since git hooks do not fire when committing with `jj`.

### ✨ Commit Messages (Gitmoji Style)

We follow [Gitmoji](https://gitmoji.dev/) for structured commit messages. Each commit should start with an emoji that
represents the change type. Example:

```shell
git commit -m "✨ Add new feature"
```

### 🤖 Continuous Integration

Every push triggers the following GitHub Actions workflows:

- **Tests** (`tests.yml`) — builds the project and runs `cargo test`.
- **Pre-commit Checks** (`pre_commit_checks.yml`) — runs the same `pre-commit` hooks from `.pre-commit-config.yaml`
  (formatting, clippy, spellcheck, etc.) and reports status via `pre-commit.ci`.
- **Requirements** (`requirements.yml`) — validates the Doorstop requirements tree in `plm/`, fails on
  an unreviewed or suspect item, and checks the scenarios, the test items and the ADR citations.
- **Semantic Release Preview** (`semantic_release.yml`) — on `main` and `action`, runs this repo's own action
  (`uses: ./`) to preview the next semantic version from Gitmoji commits, dogfooding the tool.

Two more workflows run outside the regular push cycle:

- **Release** (`release.yaml`) — manually triggered (`workflow_dispatch`); builds a release binary, publishes a
  GitHub Release, and publishes the crate to crates.io.
- **Deploy Hugo site to Pages** (`publish_pages.yml`) — runs automatically after the **Release** workflow completes
  (or manually), and builds/deploys the `docs/` Hugo site to GitHub Pages.

### 📜 Submitting a Pull Request

1. Push your branch to your fork:

   ```shell
   git push origin feature-or-bugfix-name
   ```

2. Open a **Pull Request (PR)** on GitHub:
    - Provide a clear title and description.
    - Link any relevant issue (if applicable).
    - Request a review from maintainers.
