use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, UbtError};

// ── Built-in names (used for alias conflict detection) ─────────────────

pub const BUILTIN_COMMANDS: &[&str] = &[
    "dep.install",
    "dep.remove",
    "dep.update",
    "dep.outdated",
    "dep.list",
    "dep.audit",
    "dep.lock",
    "dep.why",
    "build",
    "start",
    "run",
    "fmt",
    "run-file",
    "exec",
    "test",
    "lint",
    "check",
    "db.migrate",
    "db.rollback",
    "db.seed",
    "db.create",
    "db.drop",
    "db.reset",
    "db.status",
    "init",
    "clean",
    "release",
    "publish",
    "tool.info",
    "tool.doctor",
    "tool.list",
    "tool.docs",
    "config.show",
    "info",
    "completions",
];

pub const BUILTIN_GROUPS: &[&str] = &["dep", "db", "tool", "config"];

// ── Config structs ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectConfig {
    pub tool: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UbtConfig {
    #[serde(default)]
    pub project: Option<ProjectConfig>,
    #[serde(default)]
    pub commands: HashMap<String, String>,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

// ── Parsing ────────────────────────────────────────────────────────────

/// Parse a TOML string into an `UbtConfig`.
pub fn parse_config(content: &str) -> Result<UbtConfig> {
    toml::from_str(content).map_err(|e| {
        let line = e.span().map(|s| {
            content
                .bytes()
                .take(s.start)
                .filter(|&b| b == b'\n')
                .count()
                + 1
        });
        UbtError::config_error(line, e.message())
    })
}

// ── Alias validation ───────────────────────────────────────────────────

/// Ensure no alias shadows a built-in command or group name.
pub fn validate_aliases(config: &UbtConfig) -> Result<()> {
    for alias in config.aliases.keys() {
        if BUILTIN_COMMANDS.contains(&alias.as_str()) {
            return Err(UbtError::AliasConflict {
                alias: alias.clone(),
                command: alias.clone(),
            });
        }
        if BUILTIN_GROUPS.contains(&alias.as_str()) {
            return Err(UbtError::AliasConflict {
                alias: alias.clone(),
                command: alias.clone(),
            });
        }
    }
    Ok(())
}

// ── Config discovery ───────────────────────────────────────────────────

/// Locate and parse `ubt.toml`, returning the config and project root.
///
/// Resolution order:
/// 1. `UBT_CONFIG` environment variable (explicit path).
/// 2. Walk upward from `start_dir` looking for `ubt.toml`.
pub fn find_config(start_dir: &Path) -> Result<Option<(UbtConfig, PathBuf)>> {
    // 1. Honour UBT_CONFIG env var
    if let Ok(config_path) = std::env::var("UBT_CONFIG") {
        let path = PathBuf::from(&config_path);
        let content = std::fs::read_to_string(&path)?;
        let config = parse_config(&content)?;
        let project_root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok(Some((config, project_root)));
    }

    // 2. Walk upward
    let mut current = start_dir.to_path_buf();
    loop {
        let candidate = current.join("ubt.toml");
        if candidate.is_file() {
            let content = std::fs::read_to_string(&candidate)?;
            let config = parse_config(&content)?;
            return Ok(Some((config, current)));
        }
        if !current.pop() {
            break;
        }
    }
    Ok(None)
}

/// Load and validate the project configuration.
pub fn load_config(start_dir: &Path) -> Result<Option<(UbtConfig, PathBuf)>> {
    match find_config(start_dir)? {
        Some((config, root)) => {
            validate_aliases(&config)?;
            Ok(Some((config, root)))
        }
        None => Ok(None),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn parse_rails_example() {
        let input = r#"
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
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.project.unwrap().tool.unwrap(), "bundler");
        assert_eq!(config.commands.len(), 12);
        assert_eq!(config.aliases.len(), 3);
    }

    #[test]
    fn parse_node_prisma_example() {
        let input = r#"
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
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.project.unwrap().tool.unwrap(), "pnpm");
        assert_eq!(config.commands.len(), 10);
        assert_eq!(config.aliases.len(), 3);
    }

    #[test]
    fn parse_minimal_config() {
        let input = "[project]\ntool = \"go\"";
        let config = parse_config(input).unwrap();
        assert_eq!(config.project.unwrap().tool.unwrap(), "go");
        assert_eq!(config.commands.len(), 0);
        assert_eq!(config.aliases.len(), 0);
    }

    #[test]
    fn parse_empty_config() {
        let config = parse_config("").unwrap();
        assert!(config.project.is_none());
        assert_eq!(config.commands.len(), 0);
        assert_eq!(config.aliases.len(), 0);
    }

    #[test]
    fn parse_invalid_toml_returns_config_error() {
        let result = parse_config("[invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, UbtError::ConfigError { .. }));
    }

    #[test]
    fn validate_alias_conflicting_with_command() {
        let mut aliases = HashMap::new();
        aliases.insert("test".to_string(), "something".to_string());
        let config = UbtConfig {
            project: None,
            commands: HashMap::new(),
            aliases,
        };
        let err = validate_aliases(&config).unwrap_err();
        match err {
            UbtError::AliasConflict { alias, command } => {
                assert_eq!(alias, "test");
                assert_eq!(command, "test");
            }
            other => panic!("expected AliasConflict, got: {other:?}"),
        }
    }

    #[test]
    fn validate_alias_conflicting_with_group() {
        let mut aliases = HashMap::new();
        aliases.insert("dep".to_string(), "something".to_string());
        let config = UbtConfig {
            project: None,
            commands: HashMap::new(),
            aliases,
        };
        let err = validate_aliases(&config).unwrap_err();
        match err {
            UbtError::AliasConflict { alias, command } => {
                assert_eq!(alias, "dep");
                assert_eq!(command, "dep");
            }
            other => panic!("expected AliasConflict, got: {other:?}"),
        }
    }

    #[test]
    fn find_config_walks_upward() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("ubt.toml"), "[project]\ntool = \"go\"").unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();

        // Ensure UBT_CONFIG is not set so the walk-up logic is exercised.
        temp_env::with_var("UBT_CONFIG", None::<&str>, || {
            let result = find_config(&nested).unwrap().unwrap();
            assert_eq!(result.0.project.unwrap().tool.unwrap(), "go");
            assert_eq!(result.1, dir.path());
        });
    }

    #[test]
    fn find_config_returns_none_when_absent() {
        let dir = TempDir::new().unwrap();

        temp_env::with_var("UBT_CONFIG", None::<&str>, || {
            let result = find_config(dir.path()).unwrap();
            assert!(result.is_none());
        });
    }

    #[test]
    fn find_config_respects_ubt_config_env() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("custom.toml");
        std::fs::write(&config_path, "[project]\ntool = \"custom\"").unwrap();

        temp_env::with_var("UBT_CONFIG", Some(config_path.to_str().unwrap()), || {
            let (config, root) = find_config(Path::new("/tmp")).unwrap().unwrap();
            assert_eq!(config.project.unwrap().tool.unwrap(), "custom");
            assert_eq!(root, dir.path());
        });
    }
}
