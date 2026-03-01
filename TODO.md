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
- [x] **Task 18:** crates.io Publishing — rename package to `ubt-cli`, add `cargo publish` step to release workflow, document one-time setup
- [ ] **Task 19:** Streamline releases with `cargo-release` — install `cargo-release`, add a `release.toml` config (`publish = false`, `tag-name = "{{version}}"`, conventional commit message), so `cargo release X.Y.Z --execute` replaces the manual 4-step bump/commit/tag/push process
- [ ] **Task 20:** Fix unsafe env var usage in tests — replace `unsafe { env::set_var }` in `config.rs` and `detect.rs` tests with `temp-env` crate or parameter injection (see `specs/IMPROVEMENTS.md` §1.1)
- [ ] **Task 21:** Fix silent glob failures in `detect.rs` — `glob_matches()` silently returns `false` on bad patterns; return `Result<bool>` and propagate the error (see `specs/IMPROVEMENTS.md` §1.2)
- [x] **Task 22:** Add Unix signal exit-code forwarding — `executor.rs:245` maps signal-killed process to exit code 1; use `ExitStatusExt::signal()` for `128 + signal` (see `specs/IMPROVEMENTS.md` §1.3)
- [ ] **Task 23:** Deduplicate CLI match arms in `cli.rs` — extract `command_parts()` helper to replace the mirrored 40-arm blocks in `parse_command_name` and `collect_remaining_args` (see `specs/IMPROVEMENTS.md` §2.1)
- [ ] **Task 24:** Cache glob pattern compilation in `detect.rs` — pre-compile `GlobBuilder` patterns at plugin load time instead of rebuilding on every detection call (see `specs/IMPROVEMENTS.md` §3.1)
- [ ] **Task 25:** Split `executor.rs` into focused submodules — create `src/executor/{resolve,expand,flags,process}.rs` to separate concerns (see `specs/IMPROVEMENTS.md` §4.1)
- [ ] **Task 26:** Split `main.rs` handlers into submodules — move `cmd_info`, `cmd_tool`, `cmd_config`, `cmd_init` to `src/commands/` (see `specs/IMPROVEMENTS.md` §4.2)
- [ ] **Task 27:** Replace `HashMap` with `IndexMap` for ordered output — commands and aliases use `HashMap`; switch to `IndexMap` for deterministic `config show` / `tool list` output (see `specs/IMPROVEMENTS.md` §5.2)
- [ ] **Task 28:** Add `Display` impls for `PluginSource` and `FlagTranslation` — improves debug messages and error context (see `specs/IMPROVEMENTS.md` §5.1)
- [x] **Task 29:** Use `#[derive(Default)]` on `GlobalFlags` — replace manual `Default` impl with derive (see `specs/IMPROVEMENTS.md` §2.2)
- [x] **Task 30:** Add unit tests for `resolve_alias()` — cover: alias found, alias not found, `{{args}}` substitution, multi-word alias (see `specs/IMPROVEMENTS.md` §6.1)
- [x] **Task 31:** Add negative integration tests — at least one `.failure()` test per major error path (unknown command, unsupported flag, invalid tool, bad config TOML) (see `specs/IMPROVEMENTS.md` §6.2)
- [ ] **Task 32:** E2E Docker tests for missing ecosystems — add Docker fixtures and tests for Java (Maven + Gradle), .NET, Yarn, Bun, and Deno (see `specs/IMPROVEMENTS.md` §6.3)
- [ ] **Task 33:** Replace verbose `.iter().find()` with `.contains()` in `config.rs` — `BUILTIN_COMMANDS.iter().find(|&&c| c == alias)` → `BUILTIN_COMMANDS.contains(&alias.as_str())` and same for `BUILTIN_GROUPS` (see `specs/IMPROVEMENTS.md` §2.3)
- [ ] **Task 34:** Use `write!` instead of `format!` + `push_str` in `executor.rs` — replace `cmd.push_str(&format!(" --{}", flag_name))` with `write!(cmd, " --{}", flag_name).unwrap()` to avoid temporary `String` allocation (see `specs/IMPROVEMENTS.md` §2.4)
- [ ] **Task 35:** Add doc comments to public items — add `///` doc comments to `resolve_command()` in `src/executor.rs`, `ResolvedPlugin` and `PluginRegistry` in `src/plugin/mod.rs` (see `specs/IMPROVEMENTS.md` §7.1)
- [ ] **Task 36:** Include span in TOML parse errors — in `src/plugin/declarative.rs:62`, include `e.span()` in the error message for better diagnostics (see `specs/IMPROVEMENTS.md` §7.2)
