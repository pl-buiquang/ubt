# ubt — Universal Build Tool

> One CLI to build, test, run, and manage dependencies across every ecosystem.

[![CI](https://github.com/pl-buiquang/ubt/actions/workflows/ci.yml/badge.svg)](https://github.com/pl-buiquang/ubt/actions/workflows/ci.yml)
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

# Open an interactive REPL / shell
ubt shell
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
| `ubt start` | Start the application (production mode) |
| `ubt watch` | Rebuild/restart on file changes |

### Testing

| Command | Description |
|---------|-------------|
| `ubt test [args]` | Run the test suite |
| `ubt test:watch` | Run tests in watch mode |
| `ubt test:coverage` | Run tests with coverage report |

### Dependencies

| Command | Description |
|---------|-------------|
| `ubt dep install` | Install / sync all dependencies |
| `ubt dep add <pkg>` | Add a new dependency |
| `ubt dep remove <pkg>` | Remove a dependency |
| `ubt dep update` | Update dependencies |
| `ubt dep list` | List installed dependencies |

### Code Quality

| Command | Description |
|---------|-------------|
| `ubt fmt` | Format source code |
| `ubt fmt:check` | Check formatting without writing |
| `ubt lint` | Run the linter |
| `ubt lint:fix` | Run linter with auto-fix |

### Utilities

| Command | Description |
|---------|-------------|
| `ubt clean` | Remove build artifacts |
| `ubt shell` | Open an interactive REPL or shell |
| `ubt info` | Show detected tool and project info |
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

---

## License

[MIT](LICENSE) © 2026 UBT Contributors
