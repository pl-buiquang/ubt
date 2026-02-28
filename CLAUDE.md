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

## Task Tracking

When completing a task from `TODO.md`, mark it as done by changing `- [ ]` to `- [x]`.
