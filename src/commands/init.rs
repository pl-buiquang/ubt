use crate::detect::detect_tool;
use crate::error::UbtError;
use crate::plugin::{PluginRegistry, ResolvedPlugin};

pub fn cmd_init() -> Result<(), UbtError> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join("ubt.toml");

    if config_path.exists() {
        println!("ubt.toml already exists at {}", config_path.display());
        return Ok(());
    }

    let registry = PluginRegistry::new()?;
    let (tool, example_cmd) = match detect_tool(None, None, &cwd, &registry) {
        Ok(detection) => {
            let example = registry
                .get(&detection.plugin_name)
                .and_then(|(plugin, source)| {
                    plugin
                        .resolve_variant(&detection.variant_name, source.clone())
                        .ok()
                })
                .and_then(|resolved| init_example_command(&resolved))
                .unwrap_or_else(|| r#"start = "your-command-here""#.to_string());
            (detection.variant_name, example)
        }
        Err(_) => ("npm".to_string(), r#"start = "npm run dev""#.to_string()),
    };

    let content = format!(
        r#"# ubt.toml — Universal Build Tool configuration

[project]
# Pin the tool/runtime. Remove this line to let ubt auto-detect.
# Supported: npm, pnpm, yarn, bun, deno, cargo, go, pip, uv, poetry, bundler
tool = "{tool}"

# Override built-in commands with project-specific shell commands.
# Available keys: build, start, test, lint, fmt, check, clean, run, exec,
#   dep.install, dep.remove, dep.update, dep.list, dep.audit, dep.outdated,
#   db.migrate, db.rollback, db.seed, db.create, db.drop, db.reset, db.status
# Use {{{{args}}}} to forward extra CLI arguments to the underlying command.
[commands]
# {example_cmd}
# ...

# Add new commands not covered by built-ins.
# Names must not conflict with built-ins (build, test, dep, db, …).
[aliases]
# hello = "echo hello world"
"#,
        tool = tool,
        example_cmd = example_cmd
    );

    std::fs::write(&config_path, &content)?;
    println!("Created {}", config_path.display());
    Ok(())
}

fn init_example_command(resolved: &ResolvedPlugin) -> Option<String> {
    let preferred = ["start", "build", "test"];
    for key in &preferred {
        if let Some(cmd) = resolved.commands.get(*key) {
            let rendered = cmd.replace("{{tool}}", &resolved.binary);
            return Some(format!(r#"{key} = "{rendered}""#));
        }
    }
    // fallback: first command alphabetically
    let mut keys: Vec<&String> = resolved.commands.keys().collect();
    keys.sort();
    keys.first().map(|key| {
        let rendered = resolved.commands[*key].replace("{{tool}}", &resolved.binary);
        format!(r#"{key} = "{rendered}""#)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{PluginSource, ResolvedPlugin};
    use std::collections::HashMap;

    fn make_resolved(binary: &str, commands: &[(&str, &str)]) -> ResolvedPlugin {
        let cmds = commands
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        ResolvedPlugin {
            name: "test".to_string(),
            description: String::new(),
            homepage: None,
            install_help: None,
            variant_name: "default".to_string(),
            binary: binary.to_string(),
            commands: cmds,
            flags: HashMap::new(),
            unsupported: HashMap::new(),
            source: PluginSource::BuiltIn,
        }
    }

    #[test]
    fn init_example_prefers_start() {
        let resolved = make_resolved(
            "node",
            &[
                ("start", "{{tool}} run dev"),
                ("build", "{{tool}} run build"),
                ("test", "{{tool}} test"),
            ],
        );
        let result = init_example_command(&resolved).unwrap();
        assert!(result.starts_with("start = "), "got: {result}");
        assert!(result.contains("node run dev"));
        assert!(!result.contains("{{tool}}"));
    }

    #[test]
    fn init_example_falls_back_to_build() {
        let resolved = make_resolved(
            "go",
            &[
                ("build", "{{tool}} build ./..."),
                ("test", "{{tool}} test ./..."),
            ],
        );
        let result = init_example_command(&resolved).unwrap();
        assert!(result.starts_with("build = "), "got: {result}");
        assert!(result.contains("go build ./..."));
    }

    #[test]
    fn init_example_falls_back_to_test() {
        let resolved = make_resolved(
            "custom",
            &[("test", "{{tool}} test"), ("check", "{{tool}} check")],
        );
        let result = init_example_command(&resolved).unwrap();
        assert!(result.starts_with("test = "), "got: {result}");
    }

    #[test]
    fn init_example_falls_back_alphabetically() {
        // No start/build/test — should pick first alphabetically: "clean" < "fmt"
        let resolved = make_resolved(
            "cargo",
            &[("fmt", "{{tool}} fmt"), ("clean", "{{tool}} clean")],
        );
        let result = init_example_command(&resolved).unwrap();
        assert!(result.starts_with("clean = "), "got: {result}");
    }

    #[test]
    fn init_example_returns_none_for_empty_commands() {
        let resolved = make_resolved("mytool", &[]);
        assert!(init_example_command(&resolved).is_none());
    }

    #[test]
    fn init_example_replaces_tool_placeholder() {
        let resolved = make_resolved("cargo", &[("test", "{{tool}} test --all")]);
        let result = init_example_command(&resolved).unwrap();
        assert!(
            !result.contains("{{tool}}"),
            "placeholder not replaced: {result}"
        );
        assert!(result.contains("cargo test --all"));
    }
}
