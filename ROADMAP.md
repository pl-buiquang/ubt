# UBT Roadmap

This document tracks planned features for releases after 1.0.0. 

---

## v1.1.0 — Scripts & Task Runner

Add a `[scripts]` table to `ubt.toml` so projects can define named shell scripts alongside their tool configuration.

- Named scripts in `ubt.toml` `[scripts]` section (e.g. `lint = "cargo clippy -- -D warnings"`)
- `ubt run <name>` to execute a named script
- Argument forwarding: `ubt run <name> -- <extra args>` appends to the script command

---

## v1.2.0 — Monorepo & Workspace Support

Allow `ubt` to operate across all packages in a workspace with a single command.

- Detect workspace configurations (Cargo workspaces, npm workspaces, Go modules, etc.)
- `ubt <cmd> --all` to run the command across all packages in the workspace
- Workspace-aware detection (per-package tool overrides respected)

---

## v1.3.0 — More Ecosystem Plugins

Expand built-in plugin coverage to additional language ecosystems.

- **Zig** (`zig build`, `zig test`, `zig run`)
- **Elixir / Mix** (`mix compile`, `mix test`, `mix run`)
- **Swift / SPM** (`swift build`, `swift test`, `swift run`)
- **Dart / Flutter** (`dart pub`, `flutter build`, `flutter test`)
- **Haskell / Cabal / Stack** (`cabal build`, `stack build`, `stack test`)

---

## v1.4.0+ — Future Ideas

Longer-term improvements under consideration. Not yet scheduled.

- **Remote plugin registry** — community-contributed plugins, `ubt plugin install <name>`
- **Project scaffolding** — `ubt new <template>` to bootstrap new projects
- **Self-update** — `ubt upgrade` to update the `ubt` binary in-place
- **Global config inheritance** — `~/.config/ubt/ubt.toml` as a base merged with project config
- **Parallel workspace execution** — run commands across workspace packages in parallel with output grouping
- **Man page generation** — ship `ubt.1` generated via `clap_mangen`
- **Dynamic shell completions** — complete alias names and script names in addition to built-in commands
