use std::collections::HashMap;
use std::path::Path;
use std::process;

use crate::cli::{Cli, ToolCommand};
use crate::config::{UbtConfig, load_config};
use crate::detect::detect_tool;
use crate::error::UbtError;
use crate::plugin::PluginRegistry;

use super::info::cmd_info;

fn cmd_doctor(
    cli: &Cli,
    config: Option<&UbtConfig>,
    project_root: &Path,
    registry: &PluginRegistry,
) -> Result<(), UbtError> {
    let quiet = cli.quiet;
    let mut any_fail = false;

    macro_rules! check_ok {
        ($msg:expr) => {
            if !quiet {
                println!("[ok]   {}", $msg);
            }
        };
    }
    macro_rules! check_warn {
        ($msg:expr) => {
            if !quiet {
                println!("[warn] {}", $msg);
            }
        };
    }
    macro_rules! check_fail {
        ($msg:expr) => {{
            any_fail = true;
            eprintln!("[fail] {}", $msg);
        }};
    }

    // ── 1. Check detected tool binary ────────────────────────────────────
    let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
    match detect_tool(cli.tool.as_deref(), config_tool, project_root, registry) {
        Err(e) => {
            check_fail!(format!("Tool detection failed: {e}"));
        }
        Ok(detection) => {
            if let Some((plugin, _)) = registry.get(&detection.plugin_name)
                && let Some(variant) = plugin.variants.get(&detection.variant_name)
            {
                match which::which(&variant.binary) {
                    Ok(path) => {
                        // Try to get version
                        let version = std::process::Command::new(&variant.binary)
                            .arg("--version")
                            .output()
                            .ok()
                            .and_then(|o| {
                                let out = if o.stdout.is_empty() {
                                    o.stderr
                                } else {
                                    o.stdout
                                };
                                String::from_utf8(out).ok()
                            })
                            .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "version unknown".to_string());
                        check_ok!(format!(
                            "{} binary: {} ({})",
                            variant.binary,
                            path.display(),
                            version
                        ));
                    }
                    Err(_) => {
                        let hint = plugin
                            .install_help
                            .as_deref()
                            .map(|h| format!(" — install: {h}"))
                            .unwrap_or_default();
                        check_fail!(format!("{} is not installed{}", variant.binary, hint));
                    }
                }
            }
        }
    }

    // ── 2. Validate ubt.toml ────────────────────────────────────────────
    let cwd = std::env::current_dir()?;
    match load_config(&cwd) {
        Err(e) => check_fail!(format!("ubt.toml parse error: {e}")),
        Ok(None) => check_warn!("No ubt.toml found in this directory tree"),
        Ok(Some((cfg, config_root))) => {
            check_ok!(format!(
                "ubt.toml valid at {}",
                config_root.join("ubt.toml").display()
            ));

            // ── 3. Verify alias targets exist ──────────────────────────
            for (alias, target) in &cfg.aliases {
                let first_word = target.split_whitespace().next().unwrap_or(target.as_str());
                let known_commands = [
                    "dep.install",
                    "dep.install_pkg",
                    "dep.remove",
                    "dep.update",
                    "dep.outdated",
                    "dep.list",
                    "dep.audit",
                    "dep.lock",
                    "dep.why",
                    "build",
                    "build.dev",
                    "start",
                    "run",
                    "run-file",
                    "exec",
                    "test",
                    "lint",
                    "fmt",
                    "check",
                    "clean",
                    "release",
                    "publish",
                    "db.migrate",
                    "db.rollback",
                    "db.seed",
                    "db.create",
                    "db.drop",
                    "db.reset",
                    "db.status",
                ];
                if known_commands.contains(&first_word)
                    || cfg.aliases.contains_key(first_word)
                    || cfg.commands.contains_key(first_word)
                {
                    check_ok!(format!("alias '{alias}' → valid target '{first_word}'"));
                } else {
                    check_warn!(format!(
                        "alias '{alias}' target '{first_word}' is not a known ubt command"
                    ));
                }
            }
        }
    }

    // ── 4. Detect plugin command conflicts ──────────────────────────────
    let mut command_owners: HashMap<&str, Vec<&str>> = HashMap::new();
    for (name, (plugin, _)) in registry.iter() {
        for cmd in plugin.commands.keys() {
            command_owners
                .entry(cmd.as_str())
                .or_default()
                .push(name.as_str());
        }
    }
    let mut conflicts: Vec<_> = command_owners
        .iter()
        .filter(|(_, owners)| owners.len() > 1)
        .collect();
    conflicts.sort_by_key(|(cmd, _)| *cmd);
    if conflicts.is_empty() {
        check_ok!("No plugin command conflicts");
    } else {
        for (cmd, owners) in conflicts {
            check_warn!(format!(
                "Command '{}' defined by multiple plugins: {}",
                cmd,
                owners.join(", ")
            ));
        }
    }

    if any_fail {
        process::exit(1);
    }

    Ok(())
}

pub fn cmd_tool(
    sub: &ToolCommand,
    cli: &Cli,
    config: Option<&UbtConfig>,
    project_root: &std::path::Path,
    registry: &PluginRegistry,
) -> Result<(), UbtError> {
    match sub {
        ToolCommand::Info => cmd_info(cli, config, project_root, registry),
        ToolCommand::Doctor => cmd_doctor(cli, config, project_root, registry),
        ToolCommand::List => {
            if !cli.quiet {
                println!("{:<12} {:<30} Variants", "Plugin", "Description");
                println!("{}", "-".repeat(70));
                let mut names: Vec<_> = registry.names();
                names.sort();
                for name in names {
                    if let Some((plugin, _)) = registry.get(name) {
                        let variants: Vec<_> = plugin.variants.keys().cloned().collect();
                        println!(
                            "{:<12} {:<30} {}",
                            plugin.name,
                            plugin.description,
                            variants.join(", ")
                        );
                    }
                }
            }
            Ok(())
        }
        ToolCommand::Docs(args) => {
            let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
            let detection = detect_tool(cli.tool.as_deref(), config_tool, project_root, registry)?;
            if let Some((plugin, _)) = registry.get(&detection.plugin_name) {
                if let Some(hp) = &plugin.homepage {
                    if args.open {
                        if let Err(e) = open::that(hp) {
                            eprintln!("ubt: could not open browser: {e}");
                            if !cli.quiet {
                                println!("{hp}");
                            }
                        }
                    } else if !cli.quiet {
                        println!("{hp}");
                    }
                } else if !cli.quiet {
                    println!("No documentation URL configured for {}", plugin.name);
                }
            }
            Ok(())
        }
    }
}
