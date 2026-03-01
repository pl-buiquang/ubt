# Project Guidelines

## Overview

Rust CLI tool. Build: `cargo`. Main deps: `clap` (CLI), `serde`/`toml` (config), `thiserror` (errors).

## Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages.

Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`, `perf`, `style`, `build`

Examples:
- `feat(cli): add --verbose flag`
- `fix(config): handle missing plugin directory`
- `refactor(detect): extract marker matching logic`

IMPORTANT: 
- DO NOT add "Co-Authored-By: ..." footer
- DO NOT use a body
- USE ONE LINE DESCRIPTION

## Branch Merging

Prefer **rebase + squash merge** over simple merges. When merging a feature branch into main:
1. Rebase the feature branch onto the latest `main`
2. Squash merge into `main` with a single conventional commit message
3. Delete the feature branch after merging

## Release Process

1. Edit `Cargo.toml`: bump `version` to the new release (e.g. `0.3.0`)
2. Run `cargo check` to update `Cargo.lock`
3. Commit: `git add Cargo.toml Cargo.lock && git commit -m "chore: bump version to 0.3.0"`
4. Tag and push: `git tag 0.3.0 && git push origin main 0.3.0`

The release workflow validates that the tag matches `Cargo.toml` version and that `cargo fmt` passes before starting any builds. If there is a mismatch, the workflow fails immediately with a clear error message.

To undo a bad tag: `git tag -d 0.3.0 && git push origin :0.3.0 && gh release delete 0.3.0 --yes`

## Pre-Commit Checklist

Before every commit and push, run all of the following and fix any failures:

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Task Tracking

When completing a task from `TODO.md`, mark it as done by changing `- [ ]` to `- [x]`.
