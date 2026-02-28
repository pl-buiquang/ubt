# UBT Implementation TODO

See [PLAN.md](specs/PLAN.md) for full task details and dependency graph.

## Tasks

- [x] **Task 1:** Project Boilerplate — Cargo project, directory structure, dependencies, `.gitignore`, empty module stubs
- [x] **Task 2:** Error Types — `UbtError` enum with `thiserror`, all variants from SPEC §10
- [x] **Task 3:** Plugin Data Model — `Plugin`, `DetectConfig`, `Variant`, `FlagTranslation`, `ResolvedPlugin` structs
- [x] **Task 4:** Plugin TOML Parsing — Serde deserialization of `.toml` plugin files, validation
- [x] **Task 5:** Plugin Loading & Registry — `PluginRegistry`, built-in plugins via `include_str!`, user/project plugin dirs
- [x] **Task 6:** Config Parsing — `ubt.toml` parsing (`[project]`, `[commands]`, `[aliases]`), `find_config()`, alias validation
- [x] **Task 7:** Detection Algorithm — CLI override → env var → config → auto-detect, glob patterns, priority resolution
- [x] **Task 8:** CLI Definition — Full clap derive structure, universal flags, `--tool` flag, command name parsing
- [x] **Task 9:** Command Resolution — Alias → config override → plugin mapping → flag translation → template assembly
- [x] **Task 10:** Process Execution — Process spawning, `exec()` on Unix, exit code forwarding, `shell-words` splitting
- [x] **Task 11:** Main Pipeline Integration — Wire everything in `main.rs`, handle `info`, `config show`, `tool` subcommands, `init`
- [x] **Task 12:** Built-in Plugin — Node — Full `node.toml` with 5 variants, all mappings, flag translations
- [x] **Task 13:** Built-in Plugin — Go — Full `go.toml` from spec appendix A
- [x] **Task 14:** Built-in Plugins — Python, Rust, Java, Dotnet, Ruby — 5 remaining plugin TOML files
- [x] **Task 15:** Shell Completions — Generate completions for bash, zsh, fish, PowerShell via `clap_complete`
- [x] **Task 16:** Polish & Edge Cases — Colored errors, `--verbose` trace, `--quiet`, `--version`, config validation
- [x] **Task 17:** E2E Docker Tests — Real builds in Docker containers (node, go, rust, python, ruby) against tiny hello-world projects
