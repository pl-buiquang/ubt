# UBT Roadmap

Planned and considered features for future releases.

---

## Monorepo & Workspace Support

Allow `ubt` to operate across all packages in a workspace with a single command.

- Detect workspace configurations (Cargo workspaces, npm workspaces, Go modules, etc.)
- `ubt <cmd> --all` to run the command across all packages in the workspace
- Workspace-aware detection (per-package tool overrides respected)

---

## More Ecosystem Plugins

Expand built-in plugin coverage to additional language ecosystems.

- **Zig** (`zig build`, `zig test`, `zig run`)
- **Elixir / Mix** (`mix compile`, `mix test`, `mix run`)
- **Swift / SPM** (`swift build`, `swift test`, `swift run`)
- **Dart / Flutter** (`dart pub`, `flutter build`, `flutter test`)
- **Haskell / Cabal / Stack** (`cabal build`, `stack build`, `stack test`)

---

## Future Ideas

Longer-term improvements under consideration.

- **Remote plugin registry** — community-contributed plugins, `ubt plugin install <name>`
- **Project scaffolding** — `ubt new <template>` to bootstrap new projects
- **Self-update** — `ubt upgrade` to update the `ubt` binary in-place
- **Global config inheritance** — `~/.config/ubt/ubt.toml` as a base merged with project config
- **Parallel workspace execution** — run commands across workspace packages in parallel with output grouping
- **Man page generation** — ship `ubt.1` generated via `clap_mangen`
- **Dynamic shell completions** — complete alias names in addition to built-in commands
