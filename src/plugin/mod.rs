pub mod declarative;

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::error::{Result, UbtError};

// ── Data Model ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DetectConfig {
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub detect_files: Vec<String>,
    pub binary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagTranslation {
    Translation(String),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginSource {
    BuiltIn,
    File(PathBuf),
}

impl fmt::Display for PluginSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginSource::BuiltIn => write!(f, "built-in"),
            PluginSource::File(path) => write!(f, "file plugin at {}", path.display()),
        }
    }
}

impl fmt::Display for FlagTranslation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlagTranslation::Translation(s) => write!(f, "{s}"),
            FlagTranslation::Unsupported => write!(f, "unsupported"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub install_help: Option<String>,
    pub priority: i32,
    pub default_variant: String,
    pub detect: DetectConfig,
    pub variants: HashMap<String, Variant>,
    pub commands: HashMap<String, String>,
    pub command_variants: HashMap<String, HashMap<String, String>>,
    pub flags: HashMap<String, HashMap<String, FlagTranslation>>,
    pub unsupported: HashMap<String, String>,
}

/// A fully resolved plugin variant ready for command execution.
/// Contains the binary, command mappings, flag translations, and source metadata.
#[derive(Debug, Clone)]
pub struct ResolvedPlugin {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub install_help: Option<String>,
    pub variant_name: String,
    pub binary: String,
    pub commands: HashMap<String, String>,
    pub flags: HashMap<String, HashMap<String, FlagTranslation>>,
    pub unsupported: HashMap<String, String>,
    pub source: PluginSource,
}

impl Plugin {
    pub fn resolve_variant(
        &self,
        variant_name: &str,
        source: PluginSource,
    ) -> Result<ResolvedPlugin> {
        let variant = self
            .variants
            .get(variant_name)
            .ok_or_else(|| UbtError::PluginLoadError {
                name: self.name.clone(),
                detail: format!("variant '{}' not found", variant_name),
            })?;

        // Start with base commands, then overlay variant-specific overrides
        let mut commands = self.commands.clone();
        if let Some(overrides) = self.command_variants.get(variant_name) {
            for (cmd, mapping) in overrides {
                commands.insert(cmd.clone(), mapping.clone());
            }
        }

        Ok(ResolvedPlugin {
            name: self.name.clone(),
            description: self.description.clone(),
            homepage: self.homepage.clone(),
            install_help: self.install_help.clone(),
            variant_name: variant_name.to_string(),
            binary: variant.binary.clone(),
            commands,
            flags: self.flags.clone(),
            unsupported: self.unsupported.clone(),
            source,
        })
    }
}

// ── Built-in Plugin Data ────────────────────────────────────────────────

const BUILTIN_GO: &str = include_str!("../../plugins/go.toml");
const BUILTIN_NODE: &str = include_str!("../../plugins/node.toml");
const BUILTIN_PYTHON: &str = include_str!("../../plugins/python.toml");
const BUILTIN_RUST: &str = include_str!("../../plugins/rust.toml");
const BUILTIN_JAVA: &str = include_str!("../../plugins/java.toml");
const BUILTIN_DOTNET: &str = include_str!("../../plugins/dotnet.toml");
const BUILTIN_RUBY: &str = include_str!("../../plugins/ruby.toml");
const BUILTIN_PHP: &str = include_str!("../../plugins/php.toml");
const BUILTIN_CPP: &str = include_str!("../../plugins/cpp.toml");

const BUILTIN_PLUGINS: &[&str] = &[
    BUILTIN_GO,
    BUILTIN_NODE,
    BUILTIN_PYTHON,
    BUILTIN_RUST,
    BUILTIN_JAVA,
    BUILTIN_DOTNET,
    BUILTIN_RUBY,
    BUILTIN_PHP,
    BUILTIN_CPP,
];

// ── Plugin Registry ─────────────────────────────────────────────────────

/// Registry of all loaded plugins (built-in and user-defined).
/// Plugins are keyed by name and paired with their source location.
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: HashMap<String, (Plugin, PluginSource)>,
}

impl PluginRegistry {
    /// Create a new registry loaded with built-in plugins.
    pub fn new() -> Result<Self> {
        let mut registry = Self {
            plugins: HashMap::new(),
        };

        for toml_str in BUILTIN_PLUGINS {
            let plugin = declarative::parse_plugin_toml(toml_str)?;
            registry
                .plugins
                .insert(plugin.name.clone(), (plugin, PluginSource::BuiltIn));
        }

        Ok(registry)
    }

    /// Load plugins from a directory. Later entries override earlier ones by name.
    pub fn load_dir(&mut self, dir: &Path, source: PluginSource) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "toml")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let content = std::fs::read_to_string(entry.path())?;
            let plugin = declarative::parse_plugin_toml(&content).map_err(|_| {
                UbtError::PluginLoadError {
                    name: entry.path().display().to_string(),
                    detail: "failed to parse plugin TOML".into(),
                }
            })?;
            let file_source = match &source {
                PluginSource::BuiltIn => PluginSource::BuiltIn,
                PluginSource::File(_) => PluginSource::File(entry.path()),
            };
            self.plugins
                .insert(plugin.name.clone(), (plugin, file_source));
        }
        Ok(())
    }

    /// Load all plugin sources in priority order (later overrides earlier):
    /// 1. Built-in (already loaded in `new()`)
    /// 2. User plugins: ~/.config/ubt/plugins/
    /// 3. UBT_PLUGIN_PATH dirs
    /// 4. Project-local: .ubt/plugins/
    pub fn load_all(&mut self, project_root: Option<&Path>) -> Result<()> {
        // User plugins
        if let Some(config_dir) = dirs::config_dir() {
            let user_dir = config_dir.join("ubt").join("plugins");
            self.load_dir(&user_dir, PluginSource::File(user_dir.clone()))?;
        }

        // UBT_PLUGIN_PATH
        if let Ok(plugin_path) = std::env::var("UBT_PLUGIN_PATH") {
            for dir in plugin_path.split(':') {
                let path = PathBuf::from(dir);
                self.load_dir(&path, PluginSource::File(path.clone()))?;
            }
        }

        // Project-local plugins
        if let Some(root) = project_root {
            let local_dir = root.join(".ubt").join("plugins");
            self.load_dir(&local_dir, PluginSource::File(local_dir.clone()))?;
        }

        Ok(())
    }

    /// Get a plugin by name.
    pub fn get(&self, name: &str) -> Option<&(Plugin, PluginSource)> {
        self.plugins.get(name)
    }

    /// Iterate over all plugins.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &(Plugin, PluginSource))> {
        self.plugins.iter()
    }

    /// Get all plugin names.
    pub fn names(&self) -> Vec<&String> {
        self.plugins.keys().collect()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_plugin() -> Plugin {
        let mut variants = HashMap::new();
        variants.insert(
            "npm".to_string(),
            Variant {
                detect_files: vec!["package-lock.json".to_string()],
                binary: "npm".to_string(),
            },
        );
        variants.insert(
            "pnpm".to_string(),
            Variant {
                detect_files: vec!["pnpm-lock.yaml".to_string()],
                binary: "pnpm".to_string(),
            },
        );

        let mut commands = HashMap::new();
        commands.insert("test".to_string(), "{{tool}} test".to_string());
        commands.insert("build".to_string(), "{{tool}} run build".to_string());
        commands.insert("exec".to_string(), "npx {{args}}".to_string());

        let mut pnpm_overrides = HashMap::new();
        pnpm_overrides.insert("exec".to_string(), "pnpm dlx {{args}}".to_string());
        let mut command_variants = HashMap::new();
        command_variants.insert("pnpm".to_string(), pnpm_overrides);

        let mut test_flags = HashMap::new();
        test_flags.insert(
            "coverage".to_string(),
            FlagTranslation::Translation("--coverage".to_string()),
        );
        test_flags.insert(
            "watch".to_string(),
            FlagTranslation::Translation("--watchAll".to_string()),
        );
        let mut flags = HashMap::new();
        flags.insert("test".to_string(), test_flags);

        let mut unsupported = HashMap::new();
        unsupported.insert(
            "dep.why".to_string(),
            "Use 'npm explain' directly".to_string(),
        );

        Plugin {
            name: "node".to_string(),
            description: "Node.js projects".to_string(),
            homepage: Some("https://nodejs.org".to_string()),
            install_help: Some("https://nodejs.org/en/download/".to_string()),
            priority: 0,
            default_variant: "npm".to_string(),
            detect: DetectConfig {
                files: vec!["package.json".to_string()],
            },
            variants,
            commands,
            command_variants,
            flags,
            unsupported,
        }
    }

    #[test]
    fn resolve_variant_merges_overrides() {
        let plugin = make_test_plugin();
        let resolved = plugin
            .resolve_variant("pnpm", PluginSource::BuiltIn)
            .unwrap();
        assert_eq!(resolved.commands["exec"], "pnpm dlx {{args}}");
        assert_eq!(resolved.commands["test"], "{{tool}} test");
    }

    #[test]
    fn resolve_variant_unknown_returns_error() {
        let plugin = make_test_plugin();
        let result = plugin.resolve_variant("nonexistent", PluginSource::BuiltIn);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn resolve_variant_carries_flags() {
        let plugin = make_test_plugin();
        let resolved = plugin
            .resolve_variant("npm", PluginSource::BuiltIn)
            .unwrap();
        assert_eq!(
            resolved.flags["test"]["coverage"],
            FlagTranslation::Translation("--coverage".to_string())
        );
    }

    #[test]
    fn resolve_variant_carries_unsupported() {
        let plugin = make_test_plugin();
        let resolved = plugin
            .resolve_variant("npm", PluginSource::BuiltIn)
            .unwrap();
        assert!(resolved.unsupported.contains_key("dep.why"));
    }

    // ── Registry tests ──────────────────────────────────────────────────

    #[test]
    fn registry_loads_builtin_plugins() {
        let registry = PluginRegistry::new().unwrap();
        assert!(registry.get("go").is_some());
        assert!(registry.get("node").is_some());
        assert!(registry.get("python").is_some());
        assert!(registry.get("rust").is_some());
        assert!(registry.get("java").is_some());
        assert!(registry.get("dotnet").is_some());
        assert!(registry.get("ruby").is_some());
        assert!(registry.get("php").is_some());
        assert!(registry.get("cpp").is_some());
    }

    #[test]
    fn registry_builtin_go_has_correct_detect() {
        let registry = PluginRegistry::new().unwrap();
        let (plugin, source) = registry.get("go").unwrap();
        assert_eq!(plugin.detect.files, vec!["go.mod"]);
        assert_eq!(*source, PluginSource::BuiltIn);
    }

    #[test]
    fn registry_builtin_node_has_variants() {
        let registry = PluginRegistry::new().unwrap();
        let (plugin, _) = registry.get("node").unwrap();
        assert_eq!(plugin.variants.len(), 5);
        assert!(plugin.variants.contains_key("npm"));
        assert!(plugin.variants.contains_key("pnpm"));
        assert!(plugin.variants.contains_key("yarn"));
        assert!(plugin.variants.contains_key("bun"));
        assert!(plugin.variants.contains_key("deno"));
    }

    #[test]
    fn registry_load_dir_adds_plugins() {
        let dir = tempfile::TempDir::new().unwrap();
        let toml_content = r#"
[plugin]
name = "custom"
[detect]
files = ["custom.txt"]
[variants.default]
binary = "custom"
"#;
        std::fs::write(dir.path().join("custom.toml"), toml_content).unwrap();

        let mut registry = PluginRegistry::new().unwrap();
        registry
            .load_dir(dir.path(), PluginSource::File(dir.path().to_path_buf()))
            .unwrap();

        assert!(registry.get("custom").is_some());
    }

    #[test]
    fn registry_load_dir_overrides_builtin() {
        let dir = tempfile::TempDir::new().unwrap();
        let toml_content = r#"
[plugin]
name = "go"
description = "Custom Go"
[detect]
files = ["go.mod"]
[variants.go]
binary = "go"
"#;
        std::fs::write(dir.path().join("go.toml"), toml_content).unwrap();

        let mut registry = PluginRegistry::new().unwrap();
        registry
            .load_dir(dir.path(), PluginSource::File(dir.path().to_path_buf()))
            .unwrap();

        let (plugin, source) = registry.get("go").unwrap();
        assert_eq!(plugin.description, "Custom Go");
        assert!(matches!(source, PluginSource::File(_)));
    }

    #[test]
    fn registry_load_dir_nonexistent_is_ok() {
        let mut registry = PluginRegistry::new().unwrap();
        let result = registry.load_dir(
            Path::new("/nonexistent/path"),
            PluginSource::File(PathBuf::from("/nonexistent")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn registry_names_returns_all() {
        let registry = PluginRegistry::new().unwrap();
        let names = registry.names();
        assert!(names.len() >= 9);
    }
}
