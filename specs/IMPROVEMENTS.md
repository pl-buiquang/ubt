# UBT — Code Improvements & Best Practices

## Overview

This document tracks code quality findings, deviations from Rust best practices, performance opportunities, and test gaps discovered during a full analysis of the `ubt-zastra` codebase. Items are grouped by category and priority. Each finding maps to a task in `TODO.md`.

---

## 1. Error Handling (🔴 High)

### 1.1 Unsafe `env::set_var` in tests

**Files:** `src/config.rs:301,314,349`, `src/detect.rs:232,237`

Tests call `unsafe { env::set_var(...) }` protected by a `Mutex`. This pattern is still race-prone in multi-threaded test runs because the unsafe contract cannot be fully enforced via a `Mutex` alone.

**Fix:** Add the `temp-env` crate (`temp-env = { version = "...", features = ["async_closure"] }`) and replace manual set/restore patterns with `temp_env::with_var(...)`. Alternatively, refactor the functions under test to accept environment values as parameters, removing the global side-effect entirely.

→ **TODO Task 20**

### 1.2 Silent glob failures in `detect.rs`

**File:** `src/detect.rs:144-150` (`glob_matches`)

Invalid glob patterns silently return `false`. The user sees "no match / tool not detected" instead of an actionable error. Any typo in a plugin's `detect.patterns` goes unnoticed.

**Fix:** Change the return type to `Result<bool, UbtError>` and propagate the error so callers can surface it (or at minimum emit a `warn!` log).

→ **TODO Task 21**

### 1.3 Signal-killed process maps to exit code 1

**File:** `src/executor.rs:245`

```rust
status.code().unwrap_or(1)
```

On Unix, `status.code()` returns `None` when the process was killed by a signal. Mapping that to `1` loses the signal information and makes `ubt` indistinguishable from a normal failure.

**Fix:** Use `std::os::unix::process::ExitStatusExt::signal()` to detect signal termination and return `128 + signal` (POSIX convention).

→ **TODO Task 22**

---

## 2. Code Duplication / Idiomatic Rust (🔴 High)

### 2.1 Mirrored 40-arm `match` blocks in `cli.rs`

**File:** `src/cli.rs:361-407` (`parse_command_name`), `src/cli.rs:457-499` (`collect_remaining_args`)

Two nearly identical `match` blocks over `Commands` enum variants. Any new command requires editing both blocks, and the two can easily drift.

**Fix:** Extract a single helper:

```rust
fn command_parts(cmd: &Commands) -> (&'static str, Option<&Vec<String>>) { ... }
```

Both `parse_command_name` and `collect_remaining_args` become one-liners delegating to this helper.

→ **TODO Task 23**

### 2.2 ~~Manual `Default` impl on `GlobalFlags`~~ ✅ RESOLVED

Struct was renamed to `UniversalFlags` and already uses `#[derive(Default)]`. No action needed.

→ **TODO Task 29** (done)

### 2.3 Verbose `contains` check

**File:** `src/config.rs:88`

```rust
BUILTIN_COMMANDS.iter().find(|&&c| c == alias)
```

**Fix:**

```rust
BUILTIN_COMMANDS.contains(&alias.as_str())
```

### 2.4 Intermediate `String` allocation in command builder

**File:** `src/executor.rs:160,176`

`format!(...)` followed by `push_str` allocates a temporary `String`.

**Fix:** Use `write!(cmd, ...)` from `std::fmt::Write` to write directly into the buffer.

---

## 3. Performance (🔴 High)

### 3.1 `GlobBuilder` rebuilt on every detection call

**File:** `src/detect.rs:143-163` (`glob_matches`)

`GlobBuilder::new(pattern).build()?.compile_matcher()` is called for every file × every pattern on every detection invocation. This is repeated work when the plugin registry is already loaded.

**Fix:** Pre-compile patterns when plugins are added to the registry and store `GlobMatcher` instances on the struct. Detection then calls the pre-compiled matcher, reducing repeated compilation to zero.

→ **TODO Task 24**

---

## 4. Module Organization (🟡 Medium)

### 4.1 `executor.rs` split — evaluated and skipped

**File:** `src/executor.rs`

Evaluated splitting into `resolve.rs`, `expand.rs`, `flags.rs`, `process.rs`. The file is ~277 lines of logic with ~360 lines of tests. Splitting into 4 submodules would be overengineering at the current project size — the file-hopping overhead outweighs any organizational benefit.

**Decision:** Intentionally skipped. Revisit if the file grows significantly.

→ **TODO Task 25** (skipped)

### 4.2 `main.rs` contains command handler logic

**File:** `src/main.rs` (~398 lines)

`cmd_info`, `cmd_tool`, `cmd_config`, and `cmd_init` are defined inline in `main.rs`, making the entry point large and hard to navigate.

**Fix:** Move each handler to `src/commands/{info,tool,config,init}.rs` and re-export from `src/commands/mod.rs`.

→ **TODO Task 26**

---

## 5. Type System & Traits (🟡 Medium)

### 5.1 Missing `Display` impls

**File:** `src/plugin/mod.rs` (`PluginSource`, `FlagTranslation`)

Neither type implements `Display`. Debug output (`{:?}`) appears in user-facing error messages.

**Fix:** Implement `Display` for both types with human-readable representations.

→ **TODO Task 28**

### 5.2 `HashMap` yields non-deterministic output

**Files:** `src/config.rs`, `src/executor.rs`

Commands and aliases are stored in `HashMap`, so `ubt config show` and `ubt tool list` display entries in random order across runs.

**Fix:** Replace `HashMap` with `IndexMap` (from the `indexmap` crate) or `BTreeMap` to guarantee stable, deterministic output.

→ **TODO Task 27**

### 5.3 `FlagTranslation` lacks hint variant (optional/deferred)

**File:** `src/plugin/mod.rs:21-24`

The `UnsupportedFlag` error variant carries no hint about what the user might do instead.

**Fix (deferred):** Add `UnsupportedWithHint(String)` variant so plugins can suggest alternatives (e.g., `"use --release instead of -O"`).

---

## 6. Test Coverage (🟡 Medium / 🔴 High)

### 6.1 `resolve_alias()` has zero unit tests — ⚠️ PARTIALLY RESOLVED

**File:** `src/executor.rs:197`

Two tests now exist (`alias_found`, `alias_not_found`). Still missing: `{{args}}` substitution and multi-word alias expansion.

**Fix:** Add the two remaining test cases.

→ **TODO Task 30**

### 6.2 ~~Integration tests only assert success~~ ✅ RESOLVED

`tests/integration.rs` now includes `.failure()` assertions (e.g. `unknown_command_errors`).

→ **TODO Task 31** (done)

### 6.3 E2E Docker tests missing several ecosystems

**File:** `tests/e2e.rs`

Current Docker fixtures cover Node (npm), Go, Rust, Python, and Ruby. Missing: Java (Maven and Gradle), .NET, Yarn, Bun, and Deno.

**Fix:** Add Docker fixture directories and test cases for each missing ecosystem.

→ **TODO Task 32**

### 6.4 No tests for signal-terminated processes

**File:** `src/executor.rs`

The signal exit-code path (see §1.3) cannot be verified without a test that sends `SIGKILL`/`SIGTERM` to a spawned process.

**Fix:** Add a unit test that spawns a process, kills it with a signal, and asserts `ubt` returns `128 + signal`.

---

## 7. Documentation & Minor Cleanups (🟢 Low)

### 7.1 Missing doc comments on public items

Public functions and types lack `///` documentation:

- `resolve_command()` in `src/executor.rs`
- `ResolvedPlugin` in `src/plugin/mod.rs`
- `PluginRegistry` in `src/plugin/mod.rs`

**Fix:** Add minimal doc comments describing purpose and parameters.

### 7.2 TOML parse errors discard span info

**File:** `src/plugin/declarative.rs:62`

TOML parse errors are converted to strings, discarding the `e.span()` position info that would help users locate the error in their plugin file.

**Fix:** Include `e.span()` in the error message: `"parse error at {:?}: {}"`.
