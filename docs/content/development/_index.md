## How to Contribute

Thank you for your interest in contributing to this project! We welcome all contributions, including bug fixes, new
features, documentation improvements, and discussions.

### 🛠 Setting Up the Project

1. **Install Rust**
   Ensure you have Rust installed. We recommend using [rustup](https://rustup.rs/):

   ```shell
   rustup update
   ```

2. **Install Pre-commit Hooks**
   This project uses `pre-commit` to enforce code quality checks automatically. Install and activate the hooks:

   ```shell
   pip install pre-commit
   pre-commit install
   ```

3. **Install Spellcheck**
   This project uses `spellcheck`. Install the tool:

    ```shell
    cargo install spellcheck
    ```

4. **Build and Test**
   Run the following commands to verify everything is working:

   ```sh
   cargo build
   cargo test
   ```

4. **Build and Test**
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
  pre-commit run --all-files
  ```

### ✨ Commit Messages (Gitmoji Style)

We follow [Gitmoji](https://gitmoji.dev/) for structured commit messages. Each commit should start with an emoji that
represents the change type. Example:

```shell
git commit -m "✨ Add new feature"
```

### 📜 Submitting a Pull Request

1. Push your branch to your fork:

   ```shell
   git push origin feature-or-bugfix-name
   ```

2. Open a **Pull Request (PR)** on GitHub:
    - Provide a clear title and description.
    - Link any relevant issue (if applicable).
    - Request a review from maintainers.

### 📬 Need Help?

If you have any questions, feel free to open
a [discussion](https://github.com/8-bit-hunters/cargo-semantic-release/discussions) or reach out via issues.
