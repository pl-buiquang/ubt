# UBT — Universal Build Tool: Specification

**Version**: 0.1.0-draft
**Status**: Draft

---

## 1. Overview

UBT (Universal Build Tool) is a command-line meta-build-tool written in Rust. It does not build anything itself — it delegates to the real underlying tool (npm, cargo, go, mvn, etc.) through a unified set of commands.

### Goals

- **One CLI to learn** regardless of project stack.
- **Auto-detection** of the underlying toolchain based on project files.
- **Consistent verbs** across ecosystems (`ubt test` works in Go, Node, Python, Java, etc.).
- **Per-project configuration** for non-standard or custom setups.
- **Plugin architecture** (declarative TOML + optional scripts) so new ecosystems are first-class citizens.
- **Excellent DX**: shell completions, colored help, clear error messages with actionable guidance.

### Non-Goals

- UBT does not replace runtime/toolchain version managers (mise, asdf, nvm, pyenv). It may detect missing tools and redirect users to install pages.
- UBT does not abstract away tool-specific flags. It translates *verbs*, not *options*. Unknown flags are forwarded to the underlying tool.
- UBT does not orchestrate monorepo workspaces (deferred). It operates based on the current working directory.

---

## 2. Terminology

| Term | Meaning |
|---|---|
| **Tool** | The underlying build/package tool (npm, go, mvn, etc.) |
| **Plugin** | A UBT extension that teaches UBT how to interact with a specific tool |
| **Command** | A UBT verb like `test`, `build`, `dep install` |
| **Mapping** | The translation from a UBT command to a concrete shell command |
| **Passthrough** | Forwarding unknown flags/args directly to the underlying tool |

---

## 3. Command Taxonomy

Commands are organized into groups. Each command maps to a concrete shell invocation defined by the active plugin or overridden in project config.

A plugin may declare a command as:
- **mapped**: has a default shell command (e.g., `test = "go test ./..."`)
- **unsupported**: explicitly not available for this tool (UBT shows a clear message)
- **unmapped**: no default; requires project-level configuration

### 3.1 — Dependency Management (`dep`)

| Command | Description |
|---|---|
| `ubt dep install [pkg...]` | Install all dependencies (no args) or add specific packages |
| `ubt dep remove <pkg...>` | Remove packages |
| `ubt dep update [pkg...]` | Update all or specific packages |
| `ubt dep outdated` | List outdated dependencies |
| `ubt dep list` | List installed dependencies |
| `ubt dep audit` | Security audit of dependencies |
| `ubt dep lock` | Regenerate/update lockfile |
| `ubt dep why <pkg>` | Explain why a dependency is installed |

When `pkg` arguments are provided to `dep install`, the plugin is responsible for translating this into an "add" operation (e.g., `npm install <pkg>` vs `go get <pkg>`).

### 3.2 — Build & Compile (`build`)

| Command | Description |
|---|---|
| `ubt build` | Build the project for production/release |
| `ubt build --dev` | Build in development/debug mode |
| `ubt build --watch` | Build and watch for changes |
| `ubt build --clean` | Clean artifacts before building |

Plugins may declare `build = unsupported` for ecosystems where building is not applicable (e.g., pure Python with no compilation step).

### 3.3 — Run

| Command | Description |
|---|---|
| `ubt start` | Start the project in dev mode (hot-reload if available) |
| `ubt run <script>` | Run a named script/task (npm scripts, Makefile targets, rake tasks) |
| `ubt run:file <file> [args...]` | Run a single file (e.g., `go run`, `python`, `ruby`) |
| `ubt exec <cmd> [args...]` | Run a command in the tool's context (e.g., `npx`, `poetry run`, `bundle exec`) |

**`start` vs `run`**: `start` is an opinionated "launch the dev server" command. `run` is a generic "execute a named task". They serve different purposes and both are needed.

### 3.4 — Testing (`test`)

| Command | Description |
|---|---|
| `ubt test` | Run all tests |
| `ubt test <pattern>` | Run tests matching a pattern or path |
| `ubt test --watch` | Run tests in watch mode |
| `ubt test --coverage` | Run tests with coverage reporting |

Flags `--watch` and `--coverage` are *universal flags* that plugins translate to tool-specific equivalents. All other flags are passed through.

### 3.5 — Code Quality

| Command | Description |
|---|---|
| `ubt lint` | Run the linter |
| `ubt lint --fix` | Run linter with auto-fix |
| `ubt fmt` | Format code |
| `ubt fmt --check` | Check formatting without modifying files |
| `ubt check` | Run lint + fmt --check + type-check (pre-commit convenience) |

Linters and formatters are often separate tools from the build tool. The plugin provides sensible defaults (e.g., `gofmt` for Go, `cargo fmt` for Rust) but project config can override (e.g., pointing `lint` to ESLint in a Node project).

### 3.6 — Database (`db`)

| Command | Description |
|---|---|
| `ubt db migrate` | Run pending migrations |
| `ubt db rollback [n]` | Rollback last (or N) migrations |
| `ubt db seed` | Seed the database |
| `ubt db create` | Create the database |
| `ubt db drop` | Drop the database (requires `--yes` or interactive confirmation) |
| `ubt db reset` | Drop + create + migrate + seed |
| `ubt db status` | Show migration status |

Database commands are almost always **unmapped** at the plugin level. They require explicit project configuration except in framework-aware plugins (Rails, Django, Laravel) which provide defaults.

### 3.7 — Project Lifecycle

| Command | Description |
|---|---|
| `ubt init` | Create a `ubt.toml` config for the current project (interactive) |
| `ubt clean` | Remove build artifacts and caches |
| `ubt release` | Build for production / create release artifacts |
| `ubt publish` | Publish package to its registry (npm, PyPI, crates.io, etc.) |

### 3.8 — Toolchain Info (`tool`)

| Command | Description |
|---|---|
| `ubt tool info` | Show detected tool, version, install location |
| `ubt tool doctor` | Verify the tool is installed and functional; suggest install steps if not |
| `ubt tool list` | List all available/installed UBT plugins |
| `ubt tool docs` | Open the underlying tool's documentation (browser or URL) |

UBT does **not** install or update the underlying tools itself. `ubt tool doctor` detects if a tool is missing and provides actionable guidance: links to install pages, `mise` / `asdf` commands, or OS package manager commands.

### 3.9 — Help & Info

| Command | Description |
|---|---|
| `ubt help [command]` | Show help for UBT or a specific command, including what it maps to |
| `ubt info` | Show detected project type, tool, config location, plugin |
| `ubt config show` | Dump the fully resolved configuration (defaults + overrides) |
| `ubt completions <shell>` | Generate shell completion scripts (bash, zsh, fish, powershell) |

---

## 4. Universal Flags

A small set of flags are recognized by UBT itself and translated by plugins into tool-specific equivalents:

| Flag | Applies to | Description |
|---|---|---|
| `--watch` | `build`, `test` | Watch mode / re-run on changes |
| `--coverage` | `test` | Enable coverage reporting |
| `--verbose` / `-v` | all | Increase output verbosity |
| `--quiet` / `-q` | all | Suppress non-essential output |
| `--dev` | `build` | Development/debug build |
| `--clean` | `build` | Clean before building |
| `--fix` | `lint` | Auto-fix issues |
| `--check` | `fmt` | Check only, don't modify |
| `--yes` / `-y` | `db drop`, `db reset`, `publish` | Skip confirmation prompts |
| `--dry-run` | `publish`, `release` | Simulate without side effects |

Any flag not in this list is forwarded to the underlying tool unchanged.

### Flag Translation

Plugins define translations for universal flags:

```toml
[flags.test]
coverage = "--cover"        # Go
# coverage = "--coverage"   # Jest (default, no translation needed)

[flags.build]
dev = "--mode=development"  # Webpack-style
# dev = "--debug"           # Go-style
```

If a universal flag has no translation defined and the flag name itself is valid for the underlying tool, it is passed through as-is. If the plugin explicitly marks a flag as `unsupported`, UBT shows an error.

---

## 5. Configuration

### 5.1 — File: `ubt.toml`

Lives at the project root. Created by `ubt init`.

```toml
# Minimal example — most fields are optional.
# Plugin defaults apply for anything not specified here.

[project]
tool = "pnpm"              # Force tool (skip auto-detection)

[commands]
start = "pnpm run dev"
build = "pnpm run build"
lint = "pnpm exec eslint ."
fmt = "pnpm exec prettier --write ."
"db.migrate" = "pnpm exec prisma migrate deploy"
"db.seed" = "pnpm exec prisma db seed"
"db.status" = "pnpm exec prisma migrate status"

[aliases]
deploy = "pnpm run deploy"
storybook = "pnpm exec storybook dev"
typecheck = "pnpm exec tsc --noEmit"
```

### 5.2 — Syntax Rules

- **`[commands]`**: Override plugin defaults. Keys are UBT command names. Nested commands use dot notation (`"db.migrate"`). Value is the shell command string to execute.
- **`[aliases]`**: Project-specific shortcut commands. Accessible as `ubt <alias>`. Cannot shadow built-in commands.
- **`[project].tool`**: Force a specific tool/plugin. Skips auto-detection.

### 5.3 — Resolution Order

Configuration is resolved in this order (last wins):

1. Plugin defaults (built-in mappings)
2. `ubt.toml` `[commands]` section
3. Environment variables (`UBT_TOOL`, `UBT_CMD_<COMMAND>`)
4. CLI flags (`--tool=...`)

### 5.4 — Environment Variables

| Variable | Description |
|---|---|
| `UBT_TOOL` | Override tool detection (equivalent to `[project].tool`) |
| `UBT_CONFIG` | Path to config file (default: `./ubt.toml`) |
| `UBT_PLUGIN_PATH` | Additional directories to search for plugins (colon-separated) |
| `UBT_NO_COLOR` | Disable colored output |
| `UBT_VERBOSE` | Enable verbose output |

---

## 6. Plugin System

### 6.1 — Plugin Types

UBT supports two plugin formats:

1. **Declarative plugins** (TOML files): Define detection rules and command mappings as static configuration. Cover the majority of use cases. Trivial to author.

2. **Script plugins** (TOML + companion script): Extend a declarative plugin with a script for dynamic behavior — listing available scripts, custom detection logic, argument transformation.

### 6.2 — Declarative Plugin Format

A plugin is a `.toml` file placed in the plugin search path.

```toml
# npm.toml — Plugin for npm/Node.js projects

[plugin]
name = "npm"
description = "Node.js projects using npm"
homepage = "https://docs.npmjs.com/"
install_help = "https://nodejs.org/en/download/"

# Tool variants within the same ecosystem.
# Each variant can override detection and commands.
default_variant = "npm"

[detect]
# Files whose presence triggers this plugin.
# First match wins among variants.
files = ["package.json"]

[variants.npm]
detect_files = ["package-lock.json"]
binary = "npm"

[variants.pnpm]
detect_files = ["pnpm-lock.yaml"]
binary = "pnpm"

[variants.yarn]
detect_files = ["yarn.lock"]
binary = "yarn"

[variants.bun]
detect_files = ["bun.lockb", "bun.lock"]
binary = "bun"

[variants.deno]
detect_files = ["deno.json", "deno.jsonc"]
binary = "deno"

# Default command mappings (using {{tool}} as placeholder for the resolved binary).
[commands]
"dep.install" = "{{tool}} install"
"dep.install_pkg" = "{{tool}} add {{args}}"
"dep.remove" = "{{tool}} remove {{args}}"
"dep.update" = "{{tool}} update {{args}}"
"dep.outdated" = "{{tool}} outdated"
"dep.list" = "{{tool}} list"
"dep.audit" = "{{tool}} audit"
build = "{{tool}} run build"
start = "{{tool}} run dev"
test = "{{tool}} test"
run = "{{tool}} run {{args}}"
exec = "npx {{args}}"
lint = "{{tool}} run lint"
fmt = "{{tool}} run format"
clean = "rm -rf node_modules dist .next .nuxt"
publish = "{{tool}} publish"

# Per-variant overrides
[commands.variants.yarn]
"dep.install_pkg" = "yarn add {{args}}"
"dep.remove" = "yarn remove {{args}}"
exec = "yarn dlx {{args}}"

[commands.variants.bun]
exec = "bunx {{args}}"

[commands.variants.deno]
"dep.install" = "deno install"
"dep.install_pkg" = "deno add {{args}}"
test = "deno test"
run = "deno task {{args}}"
exec = "deno run {{args}}"

# Universal flag translations
[flags.test]
watch = "--watchAll"
coverage = "--coverage"

[flags.build]
watch = "--watch"
dev = "--mode=development"

# Unsupported commands (show clear message instead of failing cryptically)
[unsupported]
"dep.why" = "Use 'npm explain <pkg>' directly: npm explain <pkg>"
"dep.lock" = "Delete your lockfile and run 'ubt dep install' to regenerate."
```

### 6.3 — Script Plugins

A script plugin extends a declarative plugin by providing a companion script for dynamic behavior. The script is referenced from the TOML:

```toml
[plugin]
name = "npm"
script = "npm.rhai"    # Companion script (Rhai, Lua, or shell)
```

The script can implement optional hooks:

| Hook | Purpose |
|---|---|
| `detect()` | Custom detection logic beyond file checks |
| `list_scripts()` | Return available scripts/tasks for `ubt run <TAB>` completion |
| `transform_args(cmd, args)` | Modify arguments before execution |
| `post_run(cmd, exit_code)` | Run logic after a command completes |

Script language TBD (candidates: Rhai for Rust-native embedding, shell scripts for simplicity). The hook interface should remain small and stable.

### 6.4 — Plugin Search Path

Plugins are loaded from these locations, in order:

1. **Built-in plugins**: Compiled into the UBT binary (the initial set of supported tools).
2. **User plugins**: `~/.config/ubt/plugins/`
3. **Extra paths**: Directories listed in `UBT_PLUGIN_PATH`
4. **Project-local plugins**: `.ubt/plugins/` in the project root

Later paths override earlier ones (a project-local `npm.toml` overrides the built-in).

### 6.5 — Plugin Template Variables

Available in command mapping strings:

| Variable | Description |
|---|---|
| `{{tool}}` | Resolved binary name (e.g., `pnpm`) |
| `{{args}}` | All remaining positional arguments |
| `{{file}}` | The file argument (for `run:file`) |
| `{{project_root}}` | Absolute path to the project root |

---

## 7. Detection

### 7.1 — Algorithm

When UBT is invoked, it determines the active tool through this process:

1. **CLI override**: If `--tool=X` is passed, use plugin `X`. Stop.
2. **Environment**: If `UBT_TOOL` is set, use that plugin. Stop.
3. **Config**: If `ubt.toml` exists and has `[project].tool`, use that. Stop.
4. **Auto-detection**: Walk from CWD upward to filesystem root. At each directory:
   a. Check each plugin's `detect.files` list.
   b. If a plugin matches, check its variants' `detect_files` to pick the specific variant.
   c. Collect all matches with their directory depth.
5. **Priority**: If multiple plugins match at the same depth, use plugin priority (defined in plugin TOML, default: 0, higher wins). If still tied, error with a message asking the user to set `tool` in `ubt.toml`.
6. **No match**: Show a helpful error listing available plugins and suggesting `ubt init`.

### 7.2 — Config File Location

`ubt.toml` is searched for starting from CWD walking upward, similar to how `.gitignore` or `package.json` are found. The first `ubt.toml` found is used, and its directory is considered the project root.

If no `ubt.toml` is found, auto-detection still works — UBT is usable without any config file.

---

## 8. Tool Assistance

UBT does not install or manage underlying tools. When a required tool is not found in `$PATH`, UBT provides helpful guidance:

```
Error: pnpm is not installed.

To install pnpm:
  - via npm:        npm install -g pnpm
  - via corepack:   corepack enable && corepack prepare pnpm@latest --activate
  - via mise:       mise use -g pnpm
  - via Homebrew:   brew install pnpm
  - Docs:           https://pnpm.io/installation
```

Each plugin declares an `install_help` URL and can optionally provide structured install instructions.

---

## 9. Shell Completions

UBT generates completions for bash, zsh, fish, and PowerShell via `ubt completions <shell>`.

### Static Completions

All built-in commands and subcommands are completed statically (generated from command definitions at build time or on `ubt completions` invocation).

### Dynamic Completions

For commands like `ubt run <TAB>`, UBT invokes the active plugin's `list_scripts()` hook (if available) to provide project-specific completions. Examples:

- **npm**: Parse `scripts` from `package.json`
- **make**: Parse targets from `Makefile`
- **cargo**: Parse binary/example names from `Cargo.toml`
- **rake**: Run `rake -T` and parse output

For `ubt <alias> <TAB>`, aliases from `ubt.toml` are included in completions.

---

## 10. Error Handling

### Principles

1. **Never swallow exit codes.** UBT forwards the underlying tool's exit code as its own. CI/CD depends on this.
2. **Never buffer interactive I/O.** Commands that need stdin/stdout/stderr (e.g., `npm init`, interactive prompts) must have transparent passthrough. UBT uses direct process spawning (not shell capture).
3. **Actionable errors.** Every error message must tell the user what to do next.

### Error Categories

| Situation | Message Pattern |
|---|---|
| Tool not found | `Error: <tool> is not installed.` + install guidance |
| Command unsupported | `"<cmd>" is not supported by the <plugin> plugin. <hint>` |
| Command unmapped | `No command configured for "<cmd>". Add it to ubt.toml:\n\n  [commands]\n  "<cmd>" = "your command here"` |
| Config syntax error | `Error in ubt.toml line <N>: <detail>` |
| Plugin conflict | `Multiple plugins detected: <list>. Set tool in ubt.toml:\n\n  [project]\n  tool = "<tool>"` |
| No plugin matches | `Could not detect project type. Run "ubt init" or create ubt.toml.` |

---

## 11. Execution Model

### 11.1 — Command Resolution

```
User invokes: ubt test --coverage --runInBand

1. Parse CLI:
   - command = "test"
   - universal flags = { coverage: true }
   - remaining args = ["--runInBand"]

2. Resolve plugin (detection algorithm § 7.1)
   → plugin = "npm", variant = "pnpm"

3. Resolve command mapping:
   a. Check ubt.toml [commands].test → not set
   b. Check plugin [commands].test → "{{tool}} test"
   → template = "{{tool}} test"

4. Translate universal flags:
   - coverage → plugin says "--coverage"
   → append "--coverage"

5. Expand template:
   - {{tool}} = "pnpm"
   → "pnpm test --coverage"

6. Append remaining args:
   → "pnpm test --coverage --runInBand"

7. Execute via direct process spawn (inheriting stdin/stdout/stderr).

8. Forward exit code.
```

### 11.2 — The `dep install` Split

`dep install` has two modes:

- **No arguments**: Install all project dependencies (`npm install`, `pip install -r requirements.txt`).
- **With arguments**: Add specific packages (`npm install express`, `pip install requests`).

These are semantically different operations and may map to different underlying commands. The plugin defines both `dep.install` (no args) and `dep.install_pkg` (with args). UBT dispatches to the correct one based on whether arguments are present.

---

## 12. Built-in Plugins (v1)

The following plugins ship with UBT v1:

| Plugin | Variants | Detect Files |
|---|---|---|
| **node** | npm, pnpm, yarn, bun, deno | `package.json`, lockfiles |
| **go** | go | `go.mod` |
| **python** | pip, uv, poetry | `pyproject.toml`, `requirements.txt`, `setup.py` |
| **rust** | cargo | `Cargo.toml` |
| **java** | mvn, gradle | `pom.xml`, `build.gradle`, `build.gradle.kts` |
| **dotnet** | dotnet | `*.csproj`, `*.fsproj`, `*.sln` |
| **ruby** | bundler, gem | `Gemfile` |

Each plugin provides sensible command mappings for the commands in § 3 where the underlying tool supports them.

---

## 13. Performance Considerations

- **Detection caching**: Auto-detection results can be cached in memory for the duration of a single invocation. No persistent cache file is needed initially — detection is fast (a few file-existence checks).
- **Plugin loading**: Built-in plugins are compiled in. External plugins are only loaded if their files exist on disk. This is a small number of TOML parses.
- **Process spawning**: UBT replaces itself with the child process (`exec` on Unix) when possible, avoiding an extra process in the tree. On Windows, it spawns and waits.
- **Startup target**: UBT should add less than 20ms of overhead on top of the underlying tool's startup time.

---

## 14. Future Considerations (Deferred)

These features are explicitly out of scope for v1 but anticipated for future versions:

- **Monorepo / workspace orchestration**: `ubt test -w frontend`, `ubt run --all`.
- **Config inheritance / presets**: `extends = "company-standard"`.
- **Parallel command execution**: `ubt check` running lint, fmt, and typecheck in parallel.
- **Hook system**: Pre/post hooks for commands (e.g., `pre_test = "ubt lint"`).
- **Task dependencies**: Declaring that `release` depends on `test` and `build`.
- **Remote plugin registries**: Installing third-party plugins from a central registry.
- **mise/asdf integration**: Automatically activating the right tool version before running commands.

---

## 15. Project Structure (Planned)

```
ubt/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── cli.rs               # clap command definitions
│   ├── config.rs            # ubt.toml parsing and resolution
│   ├── detect.rs            # Project type detection
│   ├── plugin/
│   │   ├── mod.rs           # Plugin trait and loading
│   │   ├── declarative.rs   # TOML plugin parser
│   │   └── script.rs        # Script plugin runner
│   ├── executor.rs          # Command template expansion and process spawning
│   ├── error.rs             # Error types and display
│   └── completions.rs       # Shell completion generation
├── plugins/                  # Built-in plugin definitions
│   ├── node.toml
│   ├── go.toml
│   ├── python.toml
│   ├── rust.toml
│   ├── java.toml
│   ├── dotnet.toml
│   └── ruby.toml
└── tests/
    ├── detection_test.rs
    ├── config_test.rs
    └── executor_test.rs
```

---

## Appendix A: Example Plugin — Go

```toml
[plugin]
name = "go"
description = "Go projects"
homepage = "https://go.dev/doc/"
install_help = "https://go.dev/dl/"
priority = 0

[detect]
files = ["go.mod"]

[variants.go]
detect_files = ["go.mod"]
binary = "go"

[commands]
"dep.install" = "{{tool}} mod download"
"dep.install_pkg" = "{{tool}} get {{args}}"
"dep.remove" = "{{tool}} mod edit -droprequire {{args}}"
"dep.update" = "{{tool}} get -u {{args}}"
"dep.list" = "{{tool}} list -m all"
"dep.lock" = "{{tool}} mod tidy"
build = "{{tool}} build ./..."
"build.dev" = "{{tool}} build -gcflags='all=-N -l' ./..."
start = "{{tool}} run ."
"run:file" = "{{tool}} run {{file}}"
test = "{{tool}} test ./..."
lint = "golangci-lint run"
fmt = "{{tool}} fmt ./..."
"fmt.check" = "gofmt -l ."
clean = "{{tool}} clean -cache"
publish = "# Go modules are published by pushing a git tag"

[flags.test]
watch = "unsupported"
coverage = "-cover"

[flags.build]
watch = "unsupported"
dev = "-gcflags='all=-N -l'"

[unsupported]
"dep.audit" = "Use 'govulncheck' directly: go install golang.org/x/vuln/cmd/govulncheck@latest && govulncheck ./..."
"dep.outdated" = "Use 'go-mod-outdated': go install github.com/psampaz/go-mod-outdated@latest && go list -u -m -json all | go-mod-outdated"
"dep.why" = "Use 'go mod why <pkg>' directly: go mod why <pkg>"
```

## Appendix B: Example `ubt.toml` — Rails Project

```toml
[project]
tool = "bundler"

[commands]
start = "bin/rails server"
test = "bin/rails test"
lint = "bundle exec rubocop"
fmt = "bundle exec rubocop -a"
"db.migrate" = "bin/rails db:migrate"
"db.rollback" = "bin/rails db:rollback STEP={{args}}"
"db.seed" = "bin/rails db:seed"
"db.create" = "bin/rails db:create"
"db.drop" = "bin/rails db:drop"
"db.reset" = "bin/rails db:reset"
"db.status" = "bin/rails db:migrate:status"
run = "bin/rails {{args}}"

[aliases]
console = "bin/rails console"
routes = "bin/rails routes"
generate = "bin/rails generate"
```

## Appendix C: Example `ubt.toml` — Node + Prisma Project

```toml
[project]
tool = "pnpm"

[commands]
start = "pnpm run dev"
build = "pnpm run build"
test = "pnpm exec vitest"
lint = "pnpm exec eslint ."
fmt = "pnpm exec prettier --write ."
"fmt.check" = "pnpm exec prettier --check ."
"db.migrate" = "pnpm exec prisma migrate deploy"
"db.seed" = "pnpm exec prisma db seed"
"db.status" = "pnpm exec prisma migrate status"
"db.reset" = "pnpm exec prisma migrate reset"

[aliases]
studio = "pnpm exec prisma studio"
generate = "pnpm exec prisma generate"
typecheck = "pnpm exec tsc --noEmit"
```
