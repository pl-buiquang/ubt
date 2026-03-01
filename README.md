# ubt — Universal Build Tool

> One CLI to build, test, run, and manage dependencies across every ecosystem.

[![CI](https://github.com/pl-buiquang/ubt/actions/workflows/ci.yml/badge.svg)](https://github.com/pl-buiquang/ubt/actions/workflows/ci.yml)
[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=pl-buiquang_ubt&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=pl-buiquang_ubt)
[![Latest Release](https://img.shields.io/github/v/release/pl-buiquang/ubt)](https://github.com/pl-buiquang/ubt/releases/latest)
[![Crates.io](https://img.shields.io/crates/v/ubt-cli)](https://crates.io/crates/ubt-cli)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## About

`ubt` is a universal build tool that wraps npm, cargo, go, pip, maven, gradle, dotnet, bundler, and more behind a single consistent command interface. Stop memorizing per-ecosystem commands — `ubt build` does the right thing regardless of whether you're in a Node, Rust, Go, Python, Java, .NET, or Ruby project.

It auto-detects your project type from lockfiles and manifests, resolves the correct underlying tool, translates flags, and forwards the process transparently — no wrappers, no overhead.

---

## Installation

**One-liner (Linux & macOS):**

```sh
curl -fsSL https://raw.githubusercontent.com/pl-buiquang/ubt/main/install.sh | bash
```

This installs the latest release binary to `~/.local/bin`.

**Via Cargo:**

```sh
cargo install ubt-cli
```

---

## Quick Start

```sh
# Build the project
ubt build

# Run tests
ubt test

# Install / sync dependencies
ubt dep install

# Format source code
ubt fmt

# Run the project
ubt run
```

---

## Supported Ecosystems

| Ecosystem | Supported Tools |
|-----------|----------------|
| **Node.js** | npm, pnpm, yarn, bun, deno |
| **Go** | go |
| **Python** | pip, pipenv, poetry, uv |
| **Rust** | cargo |
| **Java** | Maven (`mvn`), Gradle (`gradle`/`gradlew`) |
| **.NET** | dotnet |
| **Ruby** | bundler (`bundle`) |

---

## Commands Reference

### Build & Run

| Command | Description |
|---------|-------------|
| `ubt build` | Compile / build the project |
| `ubt run [args]` | Run the project entry point |
| `ubt start` | Start the application (dev server, etc.) |
| `ubt run-file <file> [args]` | Run a file directly |
| `ubt exec <cmd> [args]` | Execute an arbitrary command via the tool |
| `ubt check [args]` | Type-check / compile-check without producing output |

### Testing

| Command | Description |
|---------|-------------|
| `ubt test [args]` | Run the test suite |
| `ubt test --watch` | Run tests in watch mode |
| `ubt test --coverage` | Run tests with coverage report |

### Dependencies

| Command | Description |
|---------|-------------|
| `ubt dep install` | Install / sync all dependencies |
| `ubt dep remove <pkg>` | Remove a dependency |
| `ubt dep update` | Update dependencies |
| `ubt dep outdated` | Show outdated dependencies |
| `ubt dep list` | List installed dependencies |
| `ubt dep audit` | Audit dependencies for vulnerabilities |
| `ubt dep lock` | Generate or update lock file |
| `ubt dep why <pkg>` | Explain why a dependency is installed |

### Code Quality

| Command | Description |
|---------|-------------|
| `ubt fmt` | Format source code |
| `ubt fmt --check` | Check formatting without modifying files |
| `ubt lint` | Run the linter |
| `ubt lint --fix` | Run linter with auto-fix |

### Database

| Command | Description |
|---------|-------------|
| `ubt db migrate` | Run database migrations |
| `ubt db rollback` | Rollback database migrations |
| `ubt db seed` | Seed the database |
| `ubt db create` | Create the database |
| `ubt db drop` | Drop the database |
| `ubt db reset` | Reset the database (drop + create + migrate) |
| `ubt db status` | Show migration status |

### Project Lifecycle

| Command | Description |
|---------|-------------|
| `ubt init` | Initialize a new project configuration |
| `ubt release [--dry-run]` | Create a release |
| `ubt publish [--dry-run] [-y]` | Publish a package |
| `ubt clean` | Remove build artifacts |

### Diagnostics & Config

| Command | Description |
|---------|-------------|
| `ubt info` | Show detected tool/runtime info |
| `ubt tool info` | Show detected tool information |
| `ubt tool doctor` | Run diagnostic checks |
| `ubt tool list` | List available tools/plugins |
| `ubt tool docs` | Open tool documentation |
| `ubt config show` | Show current configuration |
| `ubt completions <shell>` | Generate shell completion script |

---

## Configuration (`ubt.toml`)

Place a `ubt.toml` in your project root to override detection or remap commands:

```toml
[project]
# Force a specific tool instead of auto-detecting
tool = "pnpm"

[commands]
# Override what a universal command maps to
build = "pnpm run build:prod"
test  = "pnpm run test:ci -- --reporter=verbose"

[aliases]
# Define project-local command shortcuts
db:migrate = "node scripts/migrate.js"
db:seed    = "node scripts/seed.js"
```

---

## Global Flags

| Flag | Env Var | Description |
|------|---------|-------------|
| `--tool <TOOL>` | `UBT_TOOL` | Force a specific underlying tool |
| `--verbose` | — | Print the resolved command before running |
| `--quiet` | — | Suppress all ubt output (pass-through only) |
| `--version` | — | Print ubt version |

**Example:**

```sh
UBT_TOOL=yarn ubt dep install
ubt --tool bun run
ubt --verbose build
```

---

## Shell Completions

Generate and install completions for your shell:

```sh
# Bash
ubt completions bash >> ~/.bash_completion

# Zsh
ubt completions zsh > ~/.zfunc/_ubt

# Fish
ubt completions fish > ~/.config/fish/completions/ubt.fish
```

---

## Plugin System

`ubt` uses TOML plugin files to define tool mappings. Built-in plugins live alongside the binary. You can extend or override them by placing `.toml` files in:

- **User plugins:** `~/.config/ubt/plugins/`
- **Project plugins:** `.ubt/plugins/` (relative to project root)

A plugin file specifies detection rules (lockfile globs, manifest names), command mappings, flag translations, and tool variants. See `plugins/node.toml` for a full example.

---

## Contributing

```sh
git clone https://github.com/pl-buiquang/ubt
cd ubt
cargo test
```

Please follow [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages.

### Releasing

1. Bump the version in `Cargo.toml` and `Cargo.lock` (`cargo check`)
2. Commit: `chore: bump version to X.Y.Z`
3. Tag and push: `git tag X.Y.Z && git push origin main X.Y.Z`

The release workflow automatically builds binaries for 5 platforms, generates a changelog, publishes a GitHub release, and publishes to crates.io. A validation step ensures the tag matches `Cargo.toml` before any builds start.

---

## License

[MIT](LICENSE) © 2026 UBT Contributors
