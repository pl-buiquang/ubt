pub mod declarative;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Result, UbtError};

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
        let variant =
            self.variants
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

    #[test]
    fn plugin_default_variant_accessible() {
        let plugin = make_test_plugin();
        assert_eq!(plugin.default_variant, "npm");
    }

    #[test]
    fn resolve_variant_empty_variants_error() {
        let mut plugin = make_test_plugin();
        plugin.variants = HashMap::new();
        let result = plugin.resolve_variant("npm", PluginSource::BuiltIn);
        assert!(result.is_err());
    }

    #[test]
    fn detect_config_files_accessible() {
        let plugin = make_test_plugin();
        assert!(plugin.detect.files.contains(&"package.json".to_string()));
    }
}
