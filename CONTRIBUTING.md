# Contributing to Neopress

Thank you for your interest in contributing to Neopress! We welcome contributions to make Neopress the fastest, most reliable, and memory-safe modern CMS available.

---

## 🤝 Before You Start

- **Code of Conduct**: Everyone taking part in the project follows our [Code of Conduct](CODE_OF_CONDUCT.md).
- **Questions & ideas**: Ask in [GitHub Discussions](https://github.com/Trooper-lab/neopress/discussions). Issues are for bugs and concrete feature requests.
- **Bugs & migration problems**: Open an [issue](https://github.com/Trooper-lab/neopress/issues/new/choose) using one of the templates. The **Migration report** template helps us improve WordPress conversion (Elementor, Divi, Gutenberg, Classic themes).
- **Security issues**: Never report these publicly. Follow [SECURITY.md](SECURITY.md).
- **Larger changes**: Open an issue or Discussion first, so we can agree on the approach before you invest time in a pull request.

---

## 🛠️ Development Setup

### 1. Prerequisites
- **Rust 1.80 or newer** (with `cargo`, `rustfmt`, and `clippy`):
  ```bash
  rustup update stable
  rustup component add clippy rustfmt
  ```
- **Git**
- *(Optional)* **Node.js** or **PHP 8+** if working on golden master generator baselines.

### 2. Fork & Clone
Fork the repository on GitHub, then clone your fork:
```bash
git clone https://github.com/<your-username>/neopress.git
cd neopress
git remote add upstream https://github.com/Trooper-lab/neopress.git
cargo build
```

---

## 📐 Architecture & Workspace Guidelines

Neopress is structured as a **Virtual Cargo Workspace**:
- **`crates/neopress-server`**: Main HTTP application, Axum routing, Rusqlite persistence, SSR templates, and WordPress REST API v2 endpoints.
- **`crates/neopress-ast`**: Unified layout AST and the Elementor, Divi, and Gutenberg transpilers.
- **`crates/neopress-transpiler`**: AI verification agent and Golden Master differential tester.
- **`crates/generated_slugify`**: Rust ports of WordPress core functions, verified against Golden Master fixtures generated from the original PHP.

### ⚠️ Code Quality Rules

1. **Zero Unchecked Panics (`.unwrap()`) in Production Code**:
   Request handlers, API endpoints, and database methods must return explicit `Result<T, AppError>`. Never call `.unwrap()` or `.expect()` inside request handling logic.
2. **Parameterized SQL Queries Only**:
   Never concatenate or format user strings directly into SQL statements. Always use `rusqlite::params![...]` or named parameters to prevent SQL injection vulnerabilities.
3. **Prevent Mutex Reentrancy Deadlocks**:
   When implementing database methods, if a function already locks `self.conn.lock()`, do not invoke other public methods on `self` that attempt to re-acquire the lock on the same thread. Create or use private helpers that receive `&Connection`.
4. **Isolated In-Memory Testing**:
   Tests must use `Database::new_in_memory()` whenever possible. Do not create lingering disk `.sqlite` files in test suites.
5. **No Real Customer Data**:
   Never commit exports, screenshots, names, emails, or phone numbers from real websites. Use made-up data and reserved domains (`example.com`, `*.example`, `*.test`) in fixtures and tests.

---

## 🧪 Testing Standards

All pull requests must pass the complete workspace test suite and lint checks:

```bash
# 1. Run all workspace tests (unit and integration)
cargo test --workspace

# 2. Verify strict Clippy compliance (0 warnings allowed)
cargo clippy --workspace --all-targets -- -D warnings

# 3. Verify standard code formatting
cargo fmt --check
```

### Adding New Endpoints
When introducing a new REST endpoint:
1. Add the handler under `crates/neopress-server/src/api/` and register the route in `crates/neopress-server/src/lib.rs`.
2. Return standard `Result<..., AppError>` for automatic WordPress error serialization.
3. Add corresponding integration tests in `crates/neopress-server/tests/api_tests.rs`.
4. Update `API.md` with request/response documentation.

---

## 📜 Contributor License Agreement (CLA)

Neopress is available under the AGPLv3 and under a [commercial license](LICENSE-COMMERCIAL.md). To keep both possible, every contributor accepts our [Contributor License Agreement](CLA.md) once. You keep the copyright to your work, and everything we merge stays available under the AGPLv3.

On your first pull request, the CLA bot posts a comment with a link to the agreement. To sign, reply with exactly:

```
I have read the CLA Document and I hereby sign the CLA
```

The check turns green, and you will not be asked again for later pull requests. If you contribute as part of your job, make sure your employer allows it.

---

## 🌿 Pull Request Process

1. **Start from an up-to-date `main`** and create a feature branch in your fork:
   ```bash
   git fetch upstream
   git checkout -b feature/my-new-feature upstream/main
   ```
2. **Commit as often as you like.** Your branch's individual commits do not have to be perfect; see "How pull requests are merged" below.
3. **Run `cargo fmt`**, Clippy, and the tests locally before pushing.
4. **Open a Pull Request against `main`** and fill in the template.
   - Write the **PR title** in [Conventional Commits](https://www.conventionalcommits.org/) format, `type(scope): summary`. For example: `feat(api): add support for wp/v2/tags query filtering`. Common types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `ci`, `chore`.
   - **Link the issue** your PR resolves with `Fixes #123` in the description. GitHub closes it automatically when the PR is merged.
5. **Make CI pass on both Linux and Windows.** When `main` moves ahead of your branch, GitHub shows an **Update branch** button on the PR. Use it (or merge `upstream/main` locally) so CI tests your change against the latest code.
6. **Sign the CLA** when the bot asks (first PR only).
7. **Review**: a maintainer reviews your PR. Push follow-up commits to the same branch to address feedback; there is no need to force-push or open a new PR.

### How pull requests are merged

- `main` only accepts changes through pull requests with passing CI.
- We use **squash merging only** (merge commits and rebase merging are disabled), so `main` keeps a linear history with one commit per pull request.
- The squash commit's **title is your PR title**, and its body lists the commits from your branch. This is why the PR title needs the Conventional Commits format.
- After the merge, your branch is **deleted automatically** in this repository. You can restore it from the PR page if you need it. Branches in your fork are not touched.

---

## 📋 PR Submission Checklist

- [ ] PR title follows Conventional Commits (`type(scope): summary`).
- [ ] Related issue linked (`Fixes #123`), if there is one.
- [ ] Code compiles cleanly with `cargo build`.
- [ ] Code formatting verified with `cargo fmt --check`.
- [ ] Clippy passes with 0 warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
- [ ] All workspace tests pass (`cargo test --workspace`).
- [ ] No panics or `.unwrap()` calls introduced in request handlers.
- [ ] No real customer data in code, fixtures, or tests.
- [ ] Documentation updated (`README.md`, `ARCHITECTURE.md`, or `API.md`) where applicable.
- [ ] CLA signed (the bot asks on your first PR).
