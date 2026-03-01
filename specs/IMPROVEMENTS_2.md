# UBT — Code Improvements & Best Practices (Round 2)

## Overview

This document tracks additional code quality findings discovered after the initial improvements round (see `specs/IMPROVEMENTS.md`). Items are grouped by category and priority. Each finding maps to a task in `TODO.md`.

---

## 1. Bug Fixes (🔴 High — required before 1.0.0)

### 1.1 `spawn_command` used instead of `execute_command` in `main.rs`

**File:** `src/main.rs` (~line 146)

The main dispatch path calls `spawn_command`, which does not `exec()` the child process on Unix. This means `ubt` stays alive as a parent process, breaking signal forwarding (Ctrl-C does not reach the child), producing an extra process in `ps`, and causing incorrect exit-code forwarding for signal-killed children.

**Fix:** Replace the `spawn_command` call with `execute_command`, which uses `exec()` on Unix (replacing the `ubt` process entirely) and falls back to `spawn` + `wait` on Windows.

→ **TODO Task 37**

### 1.2 `--quiet` flag not implemented

**File:** `src/main.rs` (~lines 141–143)

The `--quiet` flag is accepted by the CLI but has no effect. Non-error output (tool detection messages, verbose traces, informational prints) is always emitted regardless of the flag value.

**Fix:** Thread the `quiet` boolean through the main pipeline and gate all non-error `println!` / `eprintln!` calls (and log output) behind `if !flags.quiet`. Error messages must always be printed.

→ **TODO Task 38**

### 1.3 `detect_variant_literal` swallows glob errors

**File:** `src/detect.rs` (~lines 231–237)

`detect_variant_literal` iterates over glob patterns and calls `glob_matches()`. Because the return type is `bool`, any `Err` variant from `glob_matches()` is silently converted to `false`. A typo in a plugin's `detect.variants` goes unnoticed — the variant just never matches.

**Fix:** Propagate `glob_matches()` errors up through `detect_variant_literal` (return `Result<Option<String>, UbtError>`) so callers can surface the error or log a warning.

→ **TODO Task 39**

### 1.4 `PluginRegistry` uses `HashMap` — non-deterministic plugin ordering

**File:** `src/plugin/mod.rs` (~line 151)

`PluginRegistry` stores plugins in a `HashMap<String, Plugin>`. Plugin iteration order is non-deterministic, which means `ubt tool list` output changes between runs and auto-detection priority is undefined when multiple plugins match.

**Fix:** Replace `HashMap` with `IndexMap` (insertion-order preserved) so plugin registration order matches listing order and detection priority is stable.

→ **TODO Task 40**

### 1.5 `UBT_PLUGIN_PATH` split uses hardcoded `:` separator

**File:** `src/plugin/mod.rs` (~line 219)

`UBT_PLUGIN_PATH` is split on `:`, which is correct on Unix but breaks on Windows (where `;` is the path separator).

**Fix:** Use `std::path::MAIN_SEPARATOR` aware splitting — either `std::env::split_paths()` (which handles both platforms) or a `cfg!(target_os = "windows")` branch.

→ **TODO Task 41**

### 1.6 `load_dir` discards TOML error details

**File:** `src/plugin/mod.rs` (~line 189)

When `load_dir` encounters a TOML parse error it logs a generic warning and skips the file, discarding the underlying error's message and span information. Users cannot tell which plugin file is broken or why.

**Fix:** Include the error message (and span if available) in the warning: `warn!("skipping {}: {}", path.display(), e)`. Optionally surface this as a `UbtError::PluginLoad` for callers that want to treat it as fatal.

→ **TODO Task 42**

### 1.7 Plugin TOML format has no `schema_version` field

**File:** all built-in `.toml` plugin files, `src/plugin/declarative.rs`

There is no version field in the plugin TOML format. When the format evolves, `ubt` will silently misinterpret old plugins or fail with confusing errors.

**Fix:** Add an optional `schema_version` field (e.g. `schema_version = 1`) to the `Plugin` struct (default `1`). Log a warning when loading a plugin whose `schema_version` is higher than the current supported version.

→ **TODO Task 43**

---

## 2. Polish (🟡 Medium — nice-to-have for 1.0.0)

### 2.1 `tool docs --open` should open a browser

**File:** `src/commands/tool.rs` (or `src/main.rs` depending on structure)

`ubt tool docs` prints the documentation URL but does not open it. The `--open` flag is accepted but ignored.

**Fix:** Use the `open` crate (`open::that(url)?`) to launch the system browser when `--open` is passed. Gracefully fall back to printing the URL if the browser cannot be opened.

→ **TODO Task 44**

### 2.2 `--verbose` traces are sparse

**File:** `src/main.rs`, `src/executor.rs`, `src/detect.rs`

`--verbose` emits basic detection output but does not trace: alias resolution steps, flag translation decisions, config override sources (env var vs. config file vs. CLI), or the final assembled command before execution.

**Fix:** Add `debug!` / `trace!` calls at each resolution step and promote them to visible output under `--verbose`. Specifically: alias chain, per-flag translation results, config source annotations, and the final command string.

→ **TODO Task 45**

### 2.3 `tool doctor` is minimal

**File:** `src/commands/tool.rs` (or `src/main.rs`)

`ubt tool doctor` exists but only checks basic connectivity. It should verify: detected tool version matches expected range, `ubt.toml` is valid TOML with no unknown keys, no two plugins claim the same command, and all referenced alias targets exist.

**Fix:** Expand `cmd_doctor` to run each check and print a structured pass/warn/fail report per check, similar to `brew doctor` or `rustup check`.

→ **TODO Task 46**
