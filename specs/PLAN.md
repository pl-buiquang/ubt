# UBT Implementation Plan

See [SPEC.md](SPEC.md) for the full specification.

## Context

UBT (Universal Build Tool) is a Rust CLI meta-build-tool that delegates to underlying build tools (npm, cargo, go, mvn, etc.) through a unified command set. The SPEC.md is complete; the repo is empty (no code yet). This plan breaks implementation into 16 small, focused tasks, each building on prior work. Every task after Task 1 includes tests.

---

## Dependencies (Cargo.toml)

| Crate | Purpose |
|-------|---------|
| `clap` 4.x (`derive`, `env`) | CLI parsing |
| `clap_complete` 4.x | Shell completions |
| `serde` 1.x (`derive`) | Serialization |
| `toml` 0.8.x | TOML parsing |
| `thiserror` 2.x | Error derivation |
| `glob` / `globset` | Glob matching (dotnet detection) |
| `which` 7.x | Binary lookup in PATH |
| `dirs` 6.x | XDG config dirs |
| `shell-words` | Command line splitting |
| **Dev:** `tempfile`, `assert_cmd`, `predicates` | Testing |

---

## Task Dependency Graph

```
T1: Boilerplate
 ├─► T2: Error Types
 │    ├─► T3: Plugin Data Model
 │    │    ├─► T4: Plugin TOML Parsing
 │    │    │    └─► T5: Plugin Loading & Registry
 │    │    │         ├─► T7: Detection (+ T6)
 │    │    │         ├─► T12: Node Plugin
 │    │    │         ├─► T13: Go Plugin
 │    │    │         └─► T14: Remaining Plugins
 │    │    └─► T9: Command Resolution (+ T6)
 │    │         └─► T10: Process Execution
 │    └─► T6: Config Parsing
 └─► T8: CLI Definition
      └─► T15: Shell Completions

T11: Main Pipeline (needs T7, T8, T9, T10)
T16: Polish (needs all)
```

---

## Tasks

### Task 1: Project Boilerplate (no tests)

Set up Cargo project, directory structure, all dependencies, `.gitignore`. Create empty module stubs so `cargo check` passes.

**Files:**
- `Cargo.toml` — edition 2024, `[[bin]] name = "ubt"`, all deps above
- `src/main.rs` — `fn main() {}` stub
- `src/lib.rs` — `pub mod` declarations for all modules
- `src/cli.rs`, `src/config.rs`, `src/detect.rs`, `src/executor.rs`, `src/error.rs`, `src/completions.rs` — empty modules
- `src/plugin/mod.rs`, `src/plugin/declarative.rs` — empty modules
- `plugins/` — empty directory (for TOML files later)
- `.gitignore` — standard Rust (`/target`)

**Verify:** `cargo check` succeeds.

---

### Task 2: Error Types

Define `UbtError` enum with `thiserror`. Variants from SPEC §10: `ToolNotFound`, `CommandUnsupported`, `CommandUnmapped`, `ConfigError`, `PluginConflict`, `NoPluginMatch`, `PluginLoadError`, `TemplateError`, `ExecutionError`, `AliasConflict`, `Io`. Define `pub type Result<T>`.

**Files:** `src/error.rs`
**Tests:** Each variant formats to expected message string. `ToolNotFound` with/without `install_help`.

---

### Task 3: Plugin Data Model

Define in-memory plugin structs: `Plugin`, `DetectConfig`, `Variant`, `FlagTranslation` (Translation | Unsupported), `PluginSource`, `ResolvedPlugin`. Method `Plugin::resolve_variant()` that merges base commands with variant overrides.

**Files:** `src/plugin/mod.rs`
**Tests:** Variant resolution merges correctly; unknown variant errors; flags/unsupported carried over.

---

### Task 4: Plugin TOML Parsing

Serde deserialization of `.toml` plugin files into `Plugin` structs. Handle `[commands.variants.X]` nesting, `[flags.COMMAND]` sections, `"unsupported"` sentinel. Validation: name required, detect files non-empty.

**Files:** `src/plugin/declarative.rs`
**Tests:** Parse Go plugin (appendix A), Node plugin (§6.2), minimal plugin, invalid TOML errors, missing fields errors, `"unsupported"` flag parsing.

---

### Task 5: Plugin Loading & Registry

`PluginRegistry` struct. Load built-in plugins (embedded via `include_str!`), user plugins from `~/.config/ubt/plugins/`, extra paths from `UBT_PLUGIN_PATH`, project-local from `.ubt/plugins/`. Later paths override earlier by name.

**Files:** `src/plugin/mod.rs`
**Tests:** Built-in plugins load; `load_dir` adds from tempdir; override behavior; invalid TOML in dir errors.

---

### Task 6: Config Parsing (`ubt.toml`)

Parse `[project]`, `[commands]`, `[aliases]` sections. `find_config()` walks from CWD upward. Respect `UBT_CONFIG` env var. Validate aliases don't shadow built-in commands.

**Files:** `src/config.rs`
**Tests:** Parse Rails example (appendix B), Node+Prisma example (appendix C), minimal config, invalid TOML errors, alias shadowing detection, `find_config` from nested dir, `find_config` returns None.

---

### Task 7: Detection Algorithm

Implement SPEC §7: CLI override → env var → config → auto-detect (dir walk, file checks, variant matching, priority). Handle glob patterns for dotnet (`*.csproj`).

**Files:** `src/detect.rs`
**Tests (tempdir-based):** `go.mod` → Go; `package.json` + `pnpm-lock.yaml` → Node/pnpm; no lockfile → default variant; CLI override; config override; priority resolution; equal-priority conflict error; no match error; nested dir walk.

---

### Task 8: CLI Definition

Full clap derive structure for all commands from SPEC §3. Universal flags from §4. Global `--tool` flag. `parse_command_name()` → dot-notation, `collect_universal_flags()`, `collect_remaining_args()`.

**Files:** `src/cli.rs`, `src/main.rs`
**Tests:** Command name mapping for all variants; flag extraction; `ubt --help` succeeds; `ubt test --help` succeeds; `ubt dep install --help` succeeds.

---

### Task 9: Command Resolution

Resolution pipeline from SPEC §11.1: alias check → config override → unsupported check → plugin mapping → `dep.install`/`dep.install_pkg` split → flag translation → template assembly. `expand_template()` for `{{tool}}`, `{{args}}`, `{{file}}`, `{{project_root}}`.

**Files:** `src/executor.rs`
**Tests:** Config override precedence; plugin mapping fallback; unsupported → error with hint; unmapped → error; dep.install split; flag translation; flag passthrough; unsupported flag → error; alias resolution; template expansion; remaining args appended.

---

### Task 10: Process Execution

Process spawning: `exec()` on Unix, spawn-and-wait elsewhere. Stdin/stdout/stderr passthrough. Exit code forwarding. Tool-not-found detection. Use `shell-words` for command splitting.

**Files:** `src/executor.rs`
**Tests:** Command splitting (`"go test ./..."` → 3 parts); quoted args; execute `echo hello` → exit 0; nonexistent binary → `ToolNotFound`; non-zero exit code forwarding.

---

### Task 11: Main Pipeline Integration

Wire everything in `main.rs`: CLI parse → env vars → config load → plugin registry → detection → variant resolution → command resolution → template expansion → execution → exit code. Handle `info`, `config show`, `tool info/doctor/list`, `init`.

**Files:** `src/main.rs`
**Tests (assert_cmd, tempdir):** `ubt info` in Go dir; `ubt info` in Node/pnpm dir; `ubt tool list`; `ubt config show` with config file; `ubt build` in empty dir → error with guidance.

---

### Task 12: Built-in Plugin — Node

Full `node.toml` with 5 variants (npm, pnpm, yarn, bun, deno), all command mappings, variant overrides, flag translations, unsupported entries.

**Files:** `plugins/node.toml`, update `src/plugin/mod.rs` built-in list
**Tests:** Parse all variants; resolve each variant correctly; flag translations; e2e detection in tempdir with `package.json` + lockfile.

---

### Task 13: Built-in Plugin — Go

Full `go.toml` from spec appendix A.

**Files:** `plugins/go.toml`, update built-in list
**Tests:** Parse; unsupported entries; flag translations (`coverage → -cover`, `watch → unsupported`).

---

### Task 14: Built-in Plugins — Python, Rust, Java, Dotnet, Ruby

5 remaining plugin TOML files.

**Files:** `plugins/python.toml` (pip/uv/poetry), `plugins/rust.toml` (cargo), `plugins/java.toml` (mvn/gradle), `plugins/dotnet.toml` (dotnet), `plugins/ruby.toml` (bundler/gem), update built-in list
**Tests:** Each parses without errors; correct detection files; correct variant binaries; spot-check command mappings; dotnet glob detection works.

---

### Task 15: Shell Completions

Generate completions for bash, zsh, fish, PowerShell via `clap_complete`.

**Files:** `src/completions.rs`, wire in `main.rs`
**Tests:** Each shell produces non-empty output; bash output contains completion marker; zsh output contains `#compdef`.

---

### Task 16: Polish & Edge Cases

Colored error output (respect `NO_COLOR`/`UBT_NO_COLOR`). `--verbose` prints detection trace. `--quiet` suppresses UBT output. `--version` flag. Config key validation.

**Files:** `src/error.rs`, `src/main.rs`, `src/config.rs`, `src/cli.rs`
**Tests:** `UBT_NO_COLOR=1` suppresses ANSI; `--verbose` shows trace; `--version` prints version; comprehensive e2e for each ecosystem.

---

## Verification Strategy

After each task:
1. `cargo check` — compiles
2. `cargo test` — all tests pass
3. `cargo clippy` — no warnings

After Task 11+:
4. Manual e2e: create a temp dir with `go.mod`, run `ubt info`, `ubt test` (with `--dry-run` or verbose)
5. Manual e2e: create a temp dir with `package.json` + `pnpm-lock.yaml`, verify detection and command resolution

After Task 16:
6. `cargo build --release` — clean release build
7. Test in a real project directory
