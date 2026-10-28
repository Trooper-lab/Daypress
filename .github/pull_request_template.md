<!--
PR title: use Conventional Commits, e.g. "fix(editor): keep block order after undo".
It becomes the commit title on main when this PR is squash-merged.
-->

## What does this change?



## Why?

<!-- Link the issue this resolves, e.g. "Fixes #123", so it closes automatically on merge. -->

## How was it tested?



## Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] No `.unwrap()` / `.expect()` added in request handlers
- [ ] No real customer data in code, fixtures, or tests
- [ ] Docs updated where needed (`README.md`, `ARCHITECTURE.md`, `API.md`)
- [ ] I have signed the [CLA](../CLA.md) (the bot asks on your first PR)
