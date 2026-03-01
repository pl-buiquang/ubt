use serde::Deserialize;
use std::collections::HashMap;

use crate::error::{Result, UbtError};
use crate::plugin::{DetectConfig, FlagTranslation, Plugin, Variant};

/// The current plugin schema version supported by this build of ubt.
const CURRENT_SCHEMA_VERSION: u32 = 1;

// ── Raw TOML structs ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawPluginToml {
    /// Optional schema version for forward-compatible plugin loading.
    #[serde(default)]
    schema_version: Option<u32>,
    plugin: RawPluginMeta,
    detect: RawDetect,
    #[serde(default)]
    variants: HashMap<String, RawVariant>,
    #[serde(default)]
    commands: RawCommands,
    #[serde(default)]
    flags: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    unsupported: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RawPluginMeta {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    install_help: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    default_variant: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawDetect {
    files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawVariant {
    #[serde(default)]
    detect_files: Vec<String>,
    binary: String,
}

#[derive(Debug, Deserialize, Default)]
struct RawCommands {
    #[serde(flatten)]
    mappings: HashMap<String, toml::Value>,
}

// ── Parsing ─────────────────────────────────────────────────────────────

/// Parse a plugin TOML string into a `Plugin` struct.
pub fn parse_plugin_toml(content: &str) -> Result<Plugin> {
    let raw: RawPluginToml = toml::from_str(content).map_err(|e| UbtError::PluginLoadError {
        name: "<unknown>".into(),
        detail: match e.span() {
            Some(span) => format!(
                "TOML parse error at {}..{}: {}",
                span.start,
                span.end,
                e.message()
            ),
            None => format!("TOML parse error: {}", e.message()),
        },
    })?;

    // Warn if the plugin schema version is newer than what we support
    if let Some(version) = raw.schema_version
        && version > CURRENT_SCHEMA_VERSION
    {
        eprintln!(
            "ubt: warning: plugin '{}' uses schema_version {}, but this version of ubt only supports {}. Some features may not work.",
            raw.plugin.name, version, CURRENT_SCHEMA_VERSION
        );
    }

    // Validate required fields
    if raw.detect.files.is_empty() {
        return Err(UbtError::PluginLoadError {
            name: raw.plugin.name,
            detail: "detect.files must not be empty".into(),
        });
    }

    // Extract base command mappings and variant overrides from [commands]
    let mut commands = HashMap::new();
    let mut command_variants: HashMap<String, HashMap<String, String>> = HashMap::new();

    for (key, value) in &raw.commands.mappings {
        if key == "variants" {
            // [commands.variants.X] sections
            if let Some(table) = value.as_table() {
                for (variant_name, variant_cmds) in table {
                    if let Some(vcmd_table) = variant_cmds.as_table() {
                        let mut overrides = HashMap::new();
                        for (cmd_name, cmd_val) in vcmd_table {
                            if let Some(s) = cmd_val.as_str() {
                                overrides.insert(cmd_name.clone(), s.to_string());
                            }
                        }
                        command_variants.insert(variant_name.clone(), overrides);
                    }
                }
            }
        } else if let Some(s) = value.as_str() {
            commands.insert(key.clone(), s.to_string());
        }
    }

    // Parse flags — "unsupported" sentinel becomes FlagTranslation::Unsupported
    let mut flags: HashMap<String, HashMap<String, FlagTranslation>> = HashMap::new();
    for (cmd_name, flag_map) in &raw.flags {
        let mut translations = HashMap::new();
        for (flag_name, flag_value) in flag_map {
            let translation = if flag_value == "unsupported" {
                FlagTranslation::Unsupported
            } else {
                FlagTranslation::Translation(flag_value.clone())
            };
            translations.insert(flag_name.clone(), translation);
        }
        flags.insert(cmd_name.clone(), translations);
    }

    // Convert variants
    let mut variants = HashMap::new();
    for (name, raw_variant) in raw.variants {
        variants.insert(
            name,
            Variant {
                detect_files: raw_variant.detect_files,
                binary: raw_variant.binary,
            },
        );
    }

    // Determine default variant
    let default_variant = raw
        .plugin
        .default_variant
        .or_else(|| variants.keys().next().cloned())
        .unwrap_or_default();

    Ok(Plugin {
        name: raw.plugin.name,
        description: raw.plugin.description.unwrap_or_default(),
        homepage: raw.plugin.homepage,
        install_help: raw.plugin.install_help,
        priority: raw.plugin.priority.unwrap_or(0),
        default_variant,
        detect: DetectConfig {
            files: raw.detect.files,
        },
        variants,
        commands,
        command_variants,
        flags,
        unsupported: raw.unsupported,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GO_PLUGIN: &str = r##"
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
"##;

    const NODE_PLUGIN: &str = r#"
[plugin]
name = "node"
description = "Node.js projects"
homepage = "https://docs.npmjs.com/"
install_help = "https://nodejs.org/en/download/"
default_variant = "npm"

[detect]
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

[flags.test]
watch = "--watchAll"
coverage = "--coverage"

[flags.build]
watch = "--watch"
dev = "--mode=development"

[unsupported]
"dep.why" = "Use 'npm explain <pkg>' directly: npm explain <pkg>"
"dep.lock" = "Delete your lockfile and run 'ubt dep install' to regenerate."
"#;

    #[test]
    fn parse_go_plugin() {
        let plugin = parse_plugin_toml(GO_PLUGIN).unwrap();
        assert_eq!(plugin.name, "go");
        assert_eq!(plugin.description, "Go projects");
        assert_eq!(plugin.homepage.as_deref(), Some("https://go.dev/doc/"));
        assert_eq!(plugin.install_help.as_deref(), Some("https://go.dev/dl/"));
        assert_eq!(plugin.priority, 0);
        assert_eq!(plugin.detect.files, vec!["go.mod"]);
        assert_eq!(plugin.variants.len(), 1);
        assert_eq!(plugin.variants["go"].binary, "go");
    }

    #[test]
    fn go_plugin_commands() {
        let plugin = parse_plugin_toml(GO_PLUGIN).unwrap();
        assert_eq!(plugin.commands["dep.install"], "{{tool}} mod download");
        assert_eq!(plugin.commands["test"], "{{tool}} test ./...");
        assert_eq!(plugin.commands["fmt"], "{{tool}} fmt ./...");
    }

    #[test]
    fn go_plugin_unsupported_flags() {
        let plugin = parse_plugin_toml(GO_PLUGIN).unwrap();
        assert_eq!(plugin.flags["test"]["watch"], FlagTranslation::Unsupported);
        assert_eq!(
            plugin.flags["test"]["coverage"],
            FlagTranslation::Translation("-cover".to_string())
        );
    }

    #[test]
    fn go_plugin_unsupported_commands() {
        let plugin = parse_plugin_toml(GO_PLUGIN).unwrap();
        assert!(plugin.unsupported.contains_key("dep.audit"));
        assert!(plugin.unsupported.contains_key("dep.outdated"));
        assert!(plugin.unsupported.contains_key("dep.why"));
    }

    #[test]
    fn parse_node_plugin() {
        let plugin = parse_plugin_toml(NODE_PLUGIN).unwrap();
        assert_eq!(plugin.name, "node");
        assert_eq!(plugin.default_variant, "npm");
        assert_eq!(plugin.detect.files, vec!["package.json"]);
        assert_eq!(plugin.variants.len(), 5);
    }

    #[test]
    fn node_plugin_variant_overrides() {
        let plugin = parse_plugin_toml(NODE_PLUGIN).unwrap();
        assert_eq!(plugin.command_variants["yarn"]["exec"], "yarn dlx {{args}}");
        assert_eq!(plugin.command_variants["bun"]["exec"], "bunx {{args}}");
        assert_eq!(plugin.command_variants["deno"]["test"], "deno test");
    }

    #[test]
    fn node_plugin_flag_translations() {
        let plugin = parse_plugin_toml(NODE_PLUGIN).unwrap();
        assert_eq!(
            plugin.flags["test"]["watch"],
            FlagTranslation::Translation("--watchAll".to_string())
        );
        assert_eq!(
            plugin.flags["build"]["dev"],
            FlagTranslation::Translation("--mode=development".to_string())
        );
    }

    #[test]
    fn node_plugin_resolve_pnpm() {
        use crate::plugin::PluginSource;
        let plugin = parse_plugin_toml(NODE_PLUGIN).unwrap();
        let resolved = plugin
            .resolve_variant("pnpm", PluginSource::BuiltIn)
            .unwrap();
        // pnpm has no exec override, so it uses base
        assert_eq!(resolved.commands["exec"], "npx {{args}}");
        assert_eq!(resolved.binary, "pnpm");
    }

    #[test]
    fn node_plugin_resolve_yarn() {
        use crate::plugin::PluginSource;
        let plugin = parse_plugin_toml(NODE_PLUGIN).unwrap();
        let resolved = plugin
            .resolve_variant("yarn", PluginSource::BuiltIn)
            .unwrap();
        assert_eq!(resolved.commands["exec"], "yarn dlx {{args}}");
        assert_eq!(resolved.commands["dep.install_pkg"], "yarn add {{args}}");
    }

    #[test]
    fn node_plugin_resolve_deno() {
        use crate::plugin::PluginSource;
        let plugin = parse_plugin_toml(NODE_PLUGIN).unwrap();
        let resolved = plugin
            .resolve_variant("deno", PluginSource::BuiltIn)
            .unwrap();
        assert_eq!(resolved.commands["test"], "deno test");
        assert_eq!(resolved.commands["run"], "deno task {{args}}");
    }

    #[test]
    fn minimal_plugin() {
        let toml = r#"
[plugin]
name = "minimal"

[detect]
files = ["marker.txt"]

[variants.default]
binary = "tool"
"#;
        let plugin = parse_plugin_toml(toml).unwrap();
        assert_eq!(plugin.name, "minimal");
        assert_eq!(plugin.description, "");
        assert!(plugin.homepage.is_none());
        assert_eq!(plugin.priority, 0);
        assert!(plugin.commands.is_empty());
        assert!(plugin.flags.is_empty());
        assert!(plugin.unsupported.is_empty());
    }

    #[test]
    fn invalid_toml_returns_error() {
        let result = parse_plugin_toml("[invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("TOML parse error"));
    }

    #[test]
    fn missing_name_returns_error() {
        let toml = r#"
[plugin]

[detect]
files = ["foo"]
"#;
        let result = parse_plugin_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn empty_detect_files_returns_error() {
        let toml = r#"
[plugin]
name = "bad"

[detect]
files = []
"#;
        let result = parse_plugin_toml(toml);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("detect.files must not be empty")
        );
    }

    #[test]
    fn missing_detect_section_returns_error() {
        let toml = r#"
[plugin]
name = "bad"
"#;
        let result = parse_plugin_toml(toml);
        assert!(result.is_err());
    }
}
