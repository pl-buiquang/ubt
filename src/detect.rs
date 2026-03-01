use std::path::{Path, PathBuf};

use globset::GlobMatcher;

use crate::error::{Result, UbtError};
use crate::plugin::{Plugin, PluginRegistry, PluginSource};

/// Result of tool detection.
#[derive(Debug)]
pub struct DetectionResult {
    pub plugin_name: String,
    pub variant_name: String,
    pub source: PluginSource,
    pub project_root: PathBuf,
}

/// Pre-compiled glob matchers for a single plugin's detect configuration.
struct CompiledPlugin<'a> {
    plugin: &'a Plugin,
    source: &'a PluginSource,
    /// Compiled matchers for `detect.files` (None = literal, Some = glob).
    detect_matchers: Vec<Option<GlobMatcher>>,
    /// Compiled matchers for each variant's `detect_files`, keyed by variant name order.
    variant_matchers: Vec<(&'a str, Vec<Option<GlobMatcher>>)>,
}

impl<'a> CompiledPlugin<'a> {
    fn new(plugin: &'a Plugin, source: &'a PluginSource) -> Result<Self> {
        let detect_matchers = compile_patterns(&plugin.detect.files)?;

        let variant_matchers = plugin
            .variants
            .iter()
            .map(|(name, variant)| {
                compile_patterns(&variant.detect_files).map(|m| (name.as_str(), m))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            plugin,
            source,
            detect_matchers,
            variant_matchers,
        })
    }
}

/// Compile a list of file patterns into optional `GlobMatcher`s.
/// Literal patterns (no `*`) are represented as `None` (checked with `.exists()`).
/// Glob patterns are compiled to `Some(GlobMatcher)`, or an error is returned.
fn compile_patterns(patterns: &[String]) -> Result<Vec<Option<GlobMatcher>>> {
    patterns
        .iter()
        .map(|p| {
            if p.contains('*') {
                globset::GlobBuilder::new(p)
                    .literal_separator(true)
                    .build()
                    .map(|g| Some(g.compile_matcher()))
                    .map_err(|e| UbtError::InvalidGlobPattern {
                        pattern: p.clone(),
                        detail: e.to_string(),
                    })
            } else {
                Ok(None)
            }
        })
        .collect()
}

/// Build compiled plugins from the registry (pre-compiles all glob patterns once).
fn compile_registry(registry: &PluginRegistry) -> Result<Vec<CompiledPlugin<'_>>> {
    registry
        .iter()
        .map(|(_name, (plugin, source))| CompiledPlugin::new(plugin, source))
        .collect()
}

/// Detect the active tool using the SPEC §7.1 priority chain:
/// 1. CLI override (`--tool`)
/// 2. Environment variable (`UBT_TOOL`)
/// 3. Config `[project].tool`
/// 4. Auto-detection (walk CWD upward, check detect files)
pub fn detect_tool(
    cli_tool: Option<&str>,
    config_tool: Option<&str>,
    start_dir: &Path,
    registry: &PluginRegistry,
) -> Result<DetectionResult> {
    // 1. CLI override
    if let Some(tool) = cli_tool {
        return resolve_explicit_tool(tool, start_dir, registry);
    }

    // 2. UBT_TOOL env var (already handled by clap's env feature on --tool,
    //    but also check explicitly for programmatic use)
    if let Ok(tool) = std::env::var("UBT_TOOL")
        && !tool.is_empty()
    {
        return resolve_explicit_tool(&tool, start_dir, registry);
    }

    // 3. Config [project].tool
    if let Some(tool) = config_tool {
        return resolve_explicit_tool(tool, start_dir, registry);
    }

    // 4. Auto-detection — pre-compile glob patterns once before walking
    let compiled = compile_registry(registry)?;
    auto_detect(start_dir, &compiled)
}

/// Resolve an explicitly named tool (from CLI, env, or config).
/// The tool name can be either a plugin name or a variant name (e.g., "pnpm").
fn resolve_explicit_tool(
    tool: &str,
    start_dir: &Path,
    registry: &PluginRegistry,
) -> Result<DetectionResult> {
    // First check if it matches a plugin name directly
    if let Some((plugin, source)) = registry.get(tool) {
        return Ok(DetectionResult {
            plugin_name: plugin.name.clone(),
            variant_name: detect_variant_literal(plugin, start_dir)
                .unwrap_or_else(|| plugin.default_variant.clone()),
            source: source.clone(),
            project_root: start_dir.to_path_buf(),
        });
    }

    // Check if it matches a variant name within any plugin
    for (_name, (plugin, source)) in registry.iter() {
        if plugin.variants.contains_key(tool) {
            return Ok(DetectionResult {
                plugin_name: plugin.name.clone(),
                variant_name: tool.to_string(),
                source: source.clone(),
                project_root: start_dir.to_path_buf(),
            });
        }
    }

    Err(UbtError::PluginLoadError {
        name: tool.to_string(),
        detail: "no plugin or variant found with this name".into(),
    })
}

/// Auto-detect tool by walking from start_dir upward using pre-compiled matchers.
fn auto_detect(start_dir: &Path, compiled: &[CompiledPlugin<'_>]) -> Result<DetectionResult> {
    let mut current = start_dir.to_path_buf();

    loop {
        let matches = detect_at_dir(&current, compiled);
        if !matches.is_empty() {
            return resolve_matches(matches, &current);
        }
        if !current.pop() {
            break;
        }
    }

    Err(UbtError::NoPluginMatch)
}

/// A detection match at a specific directory.
#[derive(Debug)]
struct DetectMatch {
    plugin_name: String,
    variant_name: String,
    priority: i32,
    source: PluginSource,
}

/// Check all plugins for matches in the given directory using pre-compiled matchers.
fn detect_at_dir(dir: &Path, compiled: &[CompiledPlugin<'_>]) -> Vec<DetectMatch> {
    let mut matches = Vec::new();

    for cp in compiled {
        if plugin_matches_dir(cp, dir) {
            let variant = detect_variant_compiled(cp, dir)
                .unwrap_or_else(|| cp.plugin.default_variant.clone());
            matches.push(DetectMatch {
                plugin_name: cp.plugin.name.clone(),
                variant_name: variant,
                priority: cp.plugin.priority,
                source: cp.source.clone(),
            });
        }
    }

    matches
}

/// Check if a plugin's detect files are present in the given directory,
/// using pre-compiled glob matchers.
fn plugin_matches_dir(cp: &CompiledPlugin<'_>, dir: &Path) -> bool {
    cp.plugin
        .detect
        .files
        .iter()
        .zip(cp.detect_matchers.iter())
        .any(|(pattern, matcher)| match matcher {
            Some(m) => glob_matches_with(dir, m),
            None => dir.join(pattern).exists(),
        })
}

/// Detect which variant to use based on lockfile presence, using pre-compiled matchers.
fn detect_variant_compiled(cp: &CompiledPlugin<'_>, dir: &Path) -> Option<String> {
    for (variant_name, matchers) in &cp.variant_matchers {
        let variant = cp.plugin.variants.get(*variant_name)?;
        for (detect_file, matcher) in variant.detect_files.iter().zip(matchers.iter()) {
            let matched = match matcher {
                Some(m) => glob_matches_with(dir, m),
                None => dir.join(detect_file).exists(),
            };
            if matched {
                return Some((*variant_name).to_string());
            }
        }
    }
    None
}

/// Detect which variant to use based on lockfile presence (literal-only path for explicit tool).
/// Used in `resolve_explicit_tool` where we don't have pre-compiled matchers.
fn detect_variant_literal(plugin: &Plugin, dir: &Path) -> Option<String> {
    for (variant_name, variant) in &plugin.variants {
        for detect_file in &variant.detect_files {
            let matched = if detect_file.contains('*') {
                globset::GlobBuilder::new(detect_file)
                    .literal_separator(true)
                    .build()
                    .ok()
                    .map(|g| glob_matches_with(dir, &g.compile_matcher()))
                    .unwrap_or(false)
            } else {
                dir.join(detect_file).exists()
            };
            if matched {
                return Some(variant_name.clone());
            }
        }
    }
    None
}

/// Check if a pre-compiled glob matcher matches any file in the directory.
fn glob_matches_with(dir: &Path, matcher: &GlobMatcher) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };

    entries.filter_map(|e| e.ok()).any(|entry| {
        entry
            .file_name()
            .to_str()
            .map(|name| matcher.is_match(name))
            .unwrap_or(false)
    })
}

/// Resolve multiple matches using priority. Error on ties.
fn resolve_matches(matches: Vec<DetectMatch>, dir: &Path) -> Result<DetectionResult> {
    assert!(!matches.is_empty());

    if matches.len() == 1 {
        let m = matches.into_iter().next().unwrap();
        return Ok(DetectionResult {
            plugin_name: m.plugin_name,
            variant_name: m.variant_name,
            source: m.source,
            project_root: dir.to_path_buf(),
        });
    }

    // Sort by priority descending
    let mut sorted = matches;
    sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Check if the top two have the same priority
    if sorted[0].priority == sorted[1].priority {
        let plugins: Vec<_> = sorted.iter().map(|m| m.plugin_name.as_str()).collect();
        return Err(UbtError::PluginConflict {
            plugins: plugins.join(", "),
            suggested_tool: sorted[0].plugin_name.clone(),
        });
    }

    let winner = sorted.into_iter().next().unwrap();
    Ok(DetectionResult {
        plugin_name: winner.plugin_name,
        variant_name: winner.variant_name,
        source: winner.source,
        project_root: dir.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn with_clean_env<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        temp_env::with_var("UBT_TOOL", None::<&str>, f)
    }

    #[test]
    fn detect_go_project() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("go.mod"), "module example.com/foo").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "go");
            assert_eq!(result.variant_name, "go");
        });
    }

    #[test]
    fn detect_node_npm() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("package.json"), "{}").unwrap();
            std::fs::write(dir.path().join("package-lock.json"), "{}").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "node");
            assert_eq!(result.variant_name, "npm");
        });
    }

    #[test]
    fn detect_node_pnpm() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("package.json"), "{}").unwrap();
            std::fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "node");
            assert_eq!(result.variant_name, "pnpm");
        });
    }

    #[test]
    fn detect_node_default_variant_when_no_lockfile() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("package.json"), "{}").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "node");
            assert_eq!(result.variant_name, "npm");
        });
    }

    #[test]
    fn detect_rust_project() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"foo\"").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "rust");
            assert_eq!(result.variant_name, "cargo");
        });
    }

    #[test]
    fn detect_cli_override() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            // Even with go.mod present, --tool=node should win
            std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(Some("node"), None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "node");
        });
    }

    #[test]
    fn detect_config_override() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, Some("node"), dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "node");
        });
    }

    #[test]
    fn detect_variant_name_as_tool() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(Some("pnpm"), None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "node");
            assert_eq!(result.variant_name, "pnpm");
        });
    }

    #[test]
    fn detect_walks_upward() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();
            let nested = dir.path().join("a").join("b").join("c");
            std::fs::create_dir_all(&nested).unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, &nested, &registry).unwrap();

            assert_eq!(result.plugin_name, "go");
            assert_eq!(result.project_root, dir.path());
        });
    }

    #[test]
    fn detect_no_match_errors() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry);

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), UbtError::NoPluginMatch));
        });
    }

    #[test]
    fn detect_unknown_tool_errors() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(Some("nonexistent"), None, dir.path(), &registry);

            assert!(result.is_err());
        });
    }

    #[test]
    fn detect_dotnet_glob() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("MyApp.csproj"), "<Project/>").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "dotnet");
        });
    }

    #[test]
    fn detect_ruby_project() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'").unwrap();
            std::fs::write(dir.path().join("Gemfile.lock"), "").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "ruby");
            assert_eq!(result.variant_name, "bundler");
        });
    }

    #[test]
    fn detect_ruby_rails_project() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::create_dir(dir.path().join("bin")).unwrap();
            std::fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'").unwrap();
            std::fs::write(dir.path().join("bin/rails"), "#!/usr/bin/env ruby").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "ruby");
            assert_eq!(result.variant_name, "rails");
        });
    }

    #[test]
    fn detect_ruby_rails_with_lockfile() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::create_dir(dir.path().join("bin")).unwrap();
            std::fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'").unwrap();
            std::fs::write(dir.path().join("Gemfile.lock"), "").unwrap();
            std::fs::write(dir.path().join("bin/rails"), "#!/usr/bin/env ruby").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "ruby");
            assert_eq!(result.variant_name, "rails");
        });
    }

    #[test]
    fn detect_python_pip() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("requirements.txt"), "flask").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "python");
        });
    }

    #[test]
    fn detect_java_maven() {
        with_clean_env(|| {
            let dir = TempDir::new().unwrap();
            std::fs::write(dir.path().join("pom.xml"), "<project/>").unwrap();

            let registry = PluginRegistry::new().unwrap();
            let result = detect_tool(None, None, dir.path(), &registry).unwrap();

            assert_eq!(result.plugin_name, "java");
            assert_eq!(result.variant_name, "mvn");
        });
    }
}
