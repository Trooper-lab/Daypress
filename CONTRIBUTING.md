# Contributing to Neopress

Thank you for your interest in contributing to Neopress! We welcome contributions to make Neopress the fastest, most reliable, and memory-safe modern CMS available.

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
```bash
git clone https://github.com/<your-username>/Neopress.git
cd Neopress
cargo build
```

---

## 📐 Architecture & Workspace Guidelines

Neopress is structured as a **Virtual Cargo Workspace**:
- **`crates/neopress-server`**: Main HTTP application, Axum routing, Rusqlite persistence, SSR templates, and WordPress REST API v2 endpoints.
- **`crates/neopress-transpiler`**: AI verification agent and Golden Master differential tester.
- **`crates/generated_slugify`**: Certified transpiled implementations.

### ⚠️ Code Quality Rules

1. **Zero Unchecked Panics (`.unwrap()`) in Production Code**:
   Request handlers, API endpoints, and database methods must return explicit `Result<T, AppError>`. Never call `.unwrap()` or `.expect()` inside request handling logic.
2. **Parameterized SQL Queries Only**:
   Never concatenate or format user strings directly into SQL statements. Always use `rusqlite::params![...]` or named parameters to prevent SQL injection vulnerabilities.
3. **Prevent Mutex Reentrancy Deadlocks**:
   When implementing database methods, if a function already locks `self.conn.lock()`, do not invoke other public methods on `self` that attempt to re-acquire the lock on the same thread. Create or use private helpers that receive `&Connection`.
4. **Isolated In-Memory Testing**:
   Tests must use `Database::new_in_memory()` whenever possible. Do not create lingering disk `.sqlite` files in test suites.

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
1. Register the route in `crates/neopress-server/src/api.rs`.
2. Return standard `Result<..., AppError>` for automatic WordPress error serialization.
3. Add corresponding integration tests in `crates/neopress-server/tests/api_tests.rs`.
4. Update `API.md` with request/response documentation.

---

## 🌿 Pull Request Process

1. Create a feature branch:
   ```bash
   git checkout -b feature/my-new-feature
   ```
2. Commit changes with clear, descriptive commit messages:
   ```bash
   git commit -m "feat(api): add support for wp/v2/tags query filtering"
   ```
3. Run `cargo fmt` before submitting.
4. Ensure CI passes on both Linux and Windows.
5. Submit a Pull Request targeting the `main` branch.

---

## 📋 PR Submission Checklist

- [ ] Code compiles cleanly with `cargo build`.
- [ ] Code formatting verified with `cargo fmt --check`.
- [ ] Clippy passes with 0 warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
- [ ] All workspace tests pass (`cargo test --workspace`).
- [ ] No panics or `.unwrap()` calls introduced in request handlers.
- [ ] Documentation updated (`README.md`, `ARCHITECTURE.md`, or `API.md`) where applicable.
