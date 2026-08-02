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

   - [Go](https://go.dev/)
   - [Hugo](https://gohugo.io/installation/)

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
