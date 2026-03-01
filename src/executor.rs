use std::collections::HashMap;
use std::fmt::Write;

use crate::cli::UniversalFlags;
use crate::config::UbtConfig;
use crate::error::{Result, UbtError};
use crate::plugin::{FlagTranslation, ResolvedPlugin};

/// Expand template placeholders in a command string.
pub fn expand_template(
    template: &str,
    tool: &str,
    args: &str,
    file: &str,
    project_root: &str,
) -> String {
    template
        .replace("{{tool}}", tool)
        .replace("{{args}}", args)
        .replace("{{file}}", file)
        .replace("{{project_root}}", project_root)
}

/// Context for command resolution.
pub struct ResolveContext<'a> {
    pub command_name: &'a str,
    pub plugin: &'a ResolvedPlugin,
    pub config: Option<&'a UbtConfig>,
    pub flags: &'a UniversalFlags,
    pub remaining_args: &'a [String],
    pub run_script: Option<&'a str>,
    pub run_file: Option<&'a str>,
    pub project_root: &'a str,
}

/// Resolve a command to a fully expanded command string ready for execution.
///
/// Resolution pipeline (SPEC §11.1):
/// 1. Config override → check `[commands]` section
/// 2. Unsupported check → error with hint
/// 3. Plugin mapping → `dep.install`/`dep.install_pkg` split
/// 4. Flag translation → append translated flags
/// 5. Template expansion → replace `{{tool}}`, `{{args}}`, etc.
/// 6. Append remaining args
pub fn resolve_command(ctx: &ResolveContext) -> Result<String> {
    let command_name = ctx.command_name;
    let plugin = ctx.plugin;
    let remaining_args = ctx.remaining_args;

    // 1. Check config command overrides first
    if let Some(cfg) = ctx.config
        && let Some(cmd_str) = cfg.commands.get(command_name)
    {
        let args_str = remaining_args.join(" ");
        let file_str = ctx.run_file.unwrap_or("");
        let expanded = expand_template(
            cmd_str,
            &plugin.binary,
            &args_str,
            file_str,
            ctx.project_root,
        );
        let with_flags = append_flags(expanded, command_name, plugin, ctx.flags)?;
        return Ok(append_remaining_if_needed(
            &with_flags,
            remaining_args,
            cmd_str,
        ));
    }

    // 2. Check unsupported
    if let Some(hint) = plugin.unsupported.get(command_name) {
        return Err(UbtError::CommandUnsupported {
            command: command_name.to_string(),
            plugin: plugin.name.clone(),
            hint: hint.clone(),
        });
    }

    // 3. Plugin mapping with dep.install split
    let effective_command = if command_name == "dep.install" && !remaining_args.is_empty() {
        "dep.install_pkg"
    } else {
        command_name
    };

    let template =
        plugin
            .commands
            .get(effective_command)
            .ok_or_else(|| UbtError::CommandUnmapped {
                command: command_name.to_string(),
            })?;

    // 4. Build args string for template expansion
    let args_str = build_args_string(remaining_args, ctx.run_script);
    let file_str = ctx.run_file.unwrap_or("");

    // 5. Expand template
    let expanded = expand_template(
        template,
        &plugin.binary,
        &args_str,
        file_str,
        ctx.project_root,
    );

    // 6. Append translated flags
    let with_flags = append_flags(expanded, command_name, plugin, ctx.flags)?;

    // 7. Append remaining args if not already consumed by {{args}}
    Ok(append_remaining_if_needed(
        &with_flags,
        remaining_args,
        template,
    ))
}

/// Build the args string for template substitution.
fn build_args_string(remaining_args: &[String], run_script: Option<&str>) -> String {
    if let Some(script) = run_script {
        if remaining_args.is_empty() {
            script.to_string()
        } else {
            format!("{} {}", script, remaining_args.join(" "))
        }
    } else {
        remaining_args.join(" ")
    }
}

/// Append translated universal flags to the command.
fn append_flags(
    mut cmd: String,
    command_name: &str,
    plugin: &ResolvedPlugin,
    flags: &UniversalFlags,
) -> Result<String> {
    let flag_map: HashMap<&str, bool> = [
        ("watch", flags.watch),
        ("coverage", flags.coverage),
        ("dev", flags.dev),
        ("clean", flags.clean),
        ("fix", flags.fix),
        ("check", flags.check),
        ("yes", flags.yes),
        ("dry_run", flags.dry_run),
    ]
    .into();

    let translations = plugin.flags.get(command_name);

    for (flag_name, is_set) in &flag_map {
        if !is_set {
            continue;
        }
        if let Some(trans_map) = translations {
            if let Some(translation) = trans_map.get(*flag_name) {
                match translation {
                    FlagTranslation::Translation(val) => {
                        cmd.push(' ');
                        cmd.push_str(val);
                    }
                    FlagTranslation::Unsupported => {
                        return Err(UbtError::CommandUnsupported {
                            command: command_name.to_string(),
                            plugin: plugin.name.clone(),
                            hint: format!(
                                "The --{} flag is not supported for this tool.",
                                flag_name
                            ),
                        });
                    }
                }
            } else {
                // No translation defined — pass through as-is
                let _ = write!(cmd, " --{}", flag_name);
            }
        } else {
            // No flag section for this command — pass through
            let _ = write!(cmd, " --{}", flag_name);
        }
    }

    Ok(cmd)
}

/// Only append remaining args if the template did NOT contain {{args}}.
fn append_remaining_if_needed(cmd: &str, remaining_args: &[String], template: &str) -> String {
    if template.contains("{{args}}") || remaining_args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, remaining_args.join(" "))
    }
}

/// Resolve an alias from config to a command string.
pub fn resolve_alias(alias: &str, config: &UbtConfig) -> Option<String> {
    config.aliases.get(alias).cloned()
}

// ── Process Execution ───────────────────────────────────────────────────

/// Split a command string into parts using shell-words.
pub fn split_command(cmd: &str) -> Result<Vec<String>> {
    shell_words::split(cmd)
        .map_err(|e| UbtError::ExecutionError(format!("Failed to parse command: {e}")))
}

/// Execute a command string, replacing the current process on Unix.
pub fn execute_command(cmd: &str, install_help: Option<&str>) -> Result<i32> {
    let parts = split_command(cmd)?;
    if parts.is_empty() {
        return Err(UbtError::ExecutionError("empty command".into()));
    }

    let binary = &parts[0];

    // Check if binary exists in PATH
    which::which(binary).map_err(|_| UbtError::tool_not_found(binary, install_help))?;

    // On Unix: use exec to replace the process
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(binary)
            .args(&parts[1..])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .exec();
        // exec() only returns on error
        Err(UbtError::ExecutionError(format!("exec failed: {err}")))
    }

    // On non-Unix: spawn and wait
    #[cfg(not(unix))]
    {
        let status = std::process::Command::new(binary)
            .args(&parts[1..])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map_err(|e| UbtError::ExecutionError(format!("spawn failed: {e}")))?;
        Ok(status.code().unwrap_or(1))
    }
}

/// Execute a command and return its exit code (non-exec variant for testing).
pub fn spawn_command(cmd: &str, install_help: Option<&str>) -> Result<i32> {
    let parts = split_command(cmd)?;
    if parts.is_empty() {
        return Err(UbtError::ExecutionError("empty command".into()));
    }

    let binary = &parts[0];
    which::which(binary).map_err(|_| UbtError::tool_not_found(binary, install_help))?;

    let status = std::process::Command::new(binary)
        .args(&parts[1..])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| UbtError::ExecutionError(format!("spawn failed: {e}")))?;

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{FlagTranslation, PluginSource, ResolvedPlugin};

    fn make_test_plugin() -> ResolvedPlugin {
        let mut commands = HashMap::new();
        commands.insert("test".to_string(), "{{tool}} test ./...".to_string());
        commands.insert("build".to_string(), "{{tool}} build ./...".to_string());
        commands.insert(
            "dep.install".to_string(),
            "{{tool}} mod download".to_string(),
        );
        commands.insert(
            "dep.install_pkg".to_string(),
            "{{tool}} get {{args}}".to_string(),
        );
        commands.insert("run".to_string(), "{{tool}} run {{args}}".to_string());
        commands.insert("run-file".to_string(), "{{tool}} run {{file}}".to_string());
        commands.insert("fmt".to_string(), "{{tool}} fmt ./...".to_string());

        let mut test_flags = HashMap::new();
        test_flags.insert(
            "coverage".to_string(),
            FlagTranslation::Translation("-cover".to_string()),
        );
        test_flags.insert("watch".to_string(), FlagTranslation::Unsupported);

        let mut flags = HashMap::new();
        flags.insert("test".to_string(), test_flags);

        let mut unsupported = HashMap::new();
        unsupported.insert(
            "dep.audit".to_string(),
            "Use govulncheck directly".to_string(),
        );

        ResolvedPlugin {
            name: "go".to_string(),
            description: "Go projects".to_string(),
            homepage: None,
            install_help: None,
            variant_name: "go".to_string(),
            binary: "go".to_string(),
            commands,
            flags,
            unsupported,
            source: PluginSource::BuiltIn,
        }
    }

    // ── Template expansion ──────────────────────────────────────────────

    #[test]
    fn expand_template_tool() {
        let result = expand_template("{{tool}} test ./...", "go", "", "", "/project");
        assert_eq!(result, "go test ./...");
    }

    #[test]
    fn expand_template_args() {
        let result = expand_template(
            "{{tool}} get {{args}}",
            "go",
            "github.com/pkg/errors",
            "",
            "/p",
        );
        assert_eq!(result, "go get github.com/pkg/errors");
    }

    #[test]
    fn expand_template_file() {
        let result = expand_template("{{tool}} run {{file}}", "go", "", "main.go", "/p");
        assert_eq!(result, "go run main.go");
    }

    #[test]
    fn expand_template_project_root() {
        let result = expand_template(
            "cd {{project_root}} && make",
            "make",
            "",
            "",
            "/home/user/project",
        );
        assert_eq!(result, "cd /home/user/project && make");
    }

    // Helper to create a ResolveContext with common defaults
    fn ctx<'a>(
        command_name: &'a str,
        plugin: &'a ResolvedPlugin,
        flags: &'a UniversalFlags,
        remaining_args: &'a [String],
        config: Option<&'a UbtConfig>,
        run_script: Option<&'a str>,
        run_file: Option<&'a str>,
    ) -> ResolveContext<'a> {
        ResolveContext {
            command_name,
            plugin,
            config,
            flags,
            remaining_args,
            run_script,
            run_file,
            project_root: "/p",
        }
    }

    // ── Command resolution ──────────────────────────────────────────────

    #[test]
    fn resolve_basic_command() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let result = resolve_command(&ctx("test", &plugin, &flags, &[], None, None, None)).unwrap();
        assert_eq!(result, "go test ./...");
    }

    #[test]
    fn resolve_with_coverage_flag() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags {
            coverage: true,
            ..Default::default()
        };
        let result = resolve_command(&ctx("test", &plugin, &flags, &[], None, None, None)).unwrap();
        assert_eq!(result, "go test ./... -cover");
    }

    #[test]
    fn resolve_unsupported_flag_errors() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags {
            watch: true,
            ..Default::default()
        };
        let result = resolve_command(&ctx("test", &plugin, &flags, &[], None, None, None));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not supported"));
    }

    #[test]
    fn resolve_unsupported_command_errors() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let result = resolve_command(&ctx("dep.audit", &plugin, &flags, &[], None, None, None));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("govulncheck"));
    }

    #[test]
    fn resolve_unmapped_command_errors() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let result = resolve_command(&ctx("deploy", &plugin, &flags, &[], None, None, None));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UbtError::CommandUnmapped { .. }
        ));
    }

    #[test]
    fn resolve_dep_install_no_args() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let result =
            resolve_command(&ctx("dep.install", &plugin, &flags, &[], None, None, None)).unwrap();
        assert_eq!(result, "go mod download");
    }

    #[test]
    fn resolve_dep_install_with_args_splits_to_install_pkg() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let args = vec!["github.com/pkg/errors".to_string()];
        let result = resolve_command(&ctx(
            "dep.install",
            &plugin,
            &flags,
            &args,
            None,
            None,
            None,
        ))
        .unwrap();
        assert_eq!(result, "go get github.com/pkg/errors");
    }

    #[test]
    fn resolve_config_override() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let mut commands = HashMap::new();
        commands.insert("test".to_string(), "custom-test-runner".to_string());
        let config = UbtConfig {
            project: None,
            commands,
            aliases: HashMap::new(),
        };
        let result = resolve_command(&ctx(
            "test",
            &plugin,
            &flags,
            &[],
            Some(&config),
            None,
            None,
        ))
        .unwrap();
        assert_eq!(result, "custom-test-runner");
    }

    #[test]
    fn resolve_remaining_args_appended() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let args = vec!["--runInBand".to_string()];
        let result =
            resolve_command(&ctx("test", &plugin, &flags, &args, None, None, None)).unwrap();
        assert_eq!(result, "go test ./... --runInBand");
    }

    #[test]
    fn resolve_remaining_args_not_doubled_when_template_has_args() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let args = vec!["express".to_string()];
        let result = resolve_command(&ctx(
            "dep.install",
            &plugin,
            &flags,
            &args,
            None,
            None,
            None,
        ))
        .unwrap();
        assert_eq!(result, "go get express");
    }

    #[test]
    fn resolve_run_with_script() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let result =
            resolve_command(&ctx("run", &plugin, &flags, &[], None, Some("dev"), None)).unwrap();
        assert_eq!(result, "go run dev");
    }

    #[test]
    fn resolve_run_file() {
        let plugin = make_test_plugin();
        let flags = UniversalFlags::default();
        let result = resolve_command(&ctx(
            "run-file",
            &plugin,
            &flags,
            &[],
            None,
            None,
            Some("main.go"),
        ))
        .unwrap();
        assert_eq!(result, "go run main.go");
    }

    #[test]
    fn alias_found() {
        let mut aliases = HashMap::new();
        aliases.insert("t".to_string(), "custom test cmd".to_string());
        let config = UbtConfig {
            project: None,
            commands: HashMap::new(),
            aliases,
        };
        assert_eq!(
            resolve_alias("t", &config),
            Some("custom test cmd".to_string())
        );
    }

    #[test]
    fn alias_not_found() {
        let config = UbtConfig::default();
        assert_eq!(resolve_alias("nonexistent", &config), None);
    }

    // ── Command splitting ───────────────────────────────────────────────

    #[test]
    fn split_simple_command() {
        let parts = split_command("go test ./...").unwrap();
        assert_eq!(parts, vec!["go", "test", "./..."]);
    }

    #[test]
    fn split_quoted_args() {
        let parts = split_command("echo 'hello world'").unwrap();
        assert_eq!(parts, vec!["echo", "hello world"]);
    }

    #[test]
    fn split_empty_returns_empty() {
        let parts = split_command("").unwrap();
        assert!(parts.is_empty());
    }

    // ── Process execution ───────────────────────────────────────────────

    #[test]
    fn spawn_echo_exits_zero() {
        let code = spawn_command("echo hello", None).unwrap();
        assert_eq!(code, 0);
    }

    #[test]
    fn spawn_nonexistent_binary_errors() {
        let result = spawn_command("nonexistent_binary_xyz_123", None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbtError::ToolNotFound { .. }));
    }

    #[test]
    fn spawn_false_exits_nonzero() {
        let code = spawn_command("false", None).unwrap();
        assert_ne!(code, 0);
    }
}
