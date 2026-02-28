use std::process;

use clap::Parser;
use ubt_cli::cli::{
    collect_remaining_args, collect_universal_flags, parse_command_name, Cli, Command,
    ConfigCommand, RunArgs, RunFileArgs, ToolCommand,
};
use ubt_cli::completions::generate_completions;
use ubt_cli::config::load_config;
use ubt_cli::detect::detect_tool;
use ubt_cli::error::UbtError;
use ubt_cli::executor::{resolve_command, spawn_command, ResolveContext};
use ubt_cli::plugin::{PluginRegistry, ResolvedPlugin};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        if use_color() {
            eprintln!("\x1b[1;31m{e}\x1b[0m");
        } else {
            eprintln!("{e}");
        }
        process::exit(1);
    }
}

fn use_color() -> bool {
    std::env::var("NO_COLOR").is_err() && std::env::var("UBT_NO_COLOR").is_err() && atty_stderr()
}

fn atty_stderr() -> bool {
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
}

fn run(cli: Cli) -> Result<(), UbtError> {
    // Handle special subcommands that don't need detection
    match &cli.command {
        Command::Completions(args) => {
            generate_completions(args.shell, &mut std::io::stdout());
            return Ok(());
        }
        Command::Config(ConfigCommand::Show) => {
            return cmd_config_show();
        }
        Command::Init => {
            return cmd_init();
        }
        _ => {}
    }

    // Load config
    let cwd = std::env::current_dir()?;
    let config_result = load_config(&cwd)?;
    let (config, project_root) = match &config_result {
        Some((c, r)) => (Some(c), r.clone()),
        None => (None, cwd.clone()),
    };

    // Load plugin registry
    let mut registry = PluginRegistry::new()?;
    registry.load_all(Some(&project_root))?;

    // Handle aliases before tool detection
    if let Command::External(ext_args) = &cli.command {
        return cmd_alias(ext_args, config);
    }

    // Handle tool subcommands that need the registry
    match &cli.command {
        Command::Tool(sub) => {
            return cmd_tool(sub, &cli, config, &project_root, &registry);
        }
        Command::Info => {
            return cmd_info(&cli, config, &project_root, &registry);
        }
        _ => {}
    }

    // Check for alias first
    let command_name = parse_command_name(&cli.command);
    // Note: aliases are checked separately from the command enum since they'd
    // need custom parsing. For now, aliases are accessible via config commands.

    // Detect tool
    let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
    let detection = detect_tool(cli.tool.as_deref(), config_tool, &project_root, &registry)?;

    if cli.verbose {
        eprintln!(
            "ubt: detected {} (variant: {}) at {}",
            detection.plugin_name,
            detection.variant_name,
            detection.project_root.display()
        );
    }

    // Resolve the plugin variant
    let (plugin, source) =
        registry
            .get(&detection.plugin_name)
            .ok_or_else(|| UbtError::PluginLoadError {
                name: detection.plugin_name.clone(),
                detail: "plugin not found in registry".into(),
            })?;
    let resolved = plugin.resolve_variant(&detection.variant_name, source.clone())?;

    // Collect flags and remaining args
    let flags = collect_universal_flags(&cli.command);
    let remaining_args = collect_remaining_args(&cli.command);

    // Extract run script/file if applicable
    let run_script = match &cli.command {
        Command::Run(RunArgs { script, .. }) => Some(script.as_str()),
        _ => None,
    };
    let run_file = match &cli.command {
        Command::RunFile(RunFileArgs { file, .. }) => Some(file.as_str()),
        _ => None,
    };

    // Resolve the command
    let project_root_str = detection.project_root.to_string_lossy();
    let cmd_str = resolve_command(&ResolveContext {
        command_name,
        plugin: &resolved,
        config,
        flags: &flags,
        remaining_args: &remaining_args,
        run_script,
        run_file,
        project_root: &project_root_str,
    })?;

    if cli.verbose {
        eprintln!("ubt: executing: {cmd_str}");
    }

    if cli.quiet {
        // In quiet mode, suppress our own output but still run the command
    }

    // Execute
    let exit_code = spawn_command(&cmd_str, resolved.install_help.as_deref())?;
    process::exit(exit_code);
}

fn cmd_info(
    cli: &Cli,
    config: Option<&ubt_cli::config::UbtConfig>,
    project_root: &std::path::Path,
    registry: &PluginRegistry,
) -> Result<(), UbtError> {
    let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
    let detection = detect_tool(cli.tool.as_deref(), config_tool, project_root, registry)?;

    if cli.verbose {
        eprintln!(
            "ubt: detected {} (variant: {}) at {}",
            detection.plugin_name,
            detection.variant_name,
            detection.project_root.display()
        );
    }

    let (plugin, _) = registry
        .get(&detection.plugin_name)
        .ok_or(UbtError::NoPluginMatch)?;

    println!("Plugin:       {}", detection.plugin_name);
    println!("Variant:      {}", detection.variant_name);
    println!("Description:  {}", plugin.description);
    if let Some(hp) = &plugin.homepage {
        println!("Homepage:     {hp}");
    }
    println!("Project root: {}", detection.project_root.display());
    if let Some(binary) = plugin.variants.get(&detection.variant_name) {
        println!("Binary:       {}", binary.binary);
        match which::which(&binary.binary) {
            Ok(path) => println!("Binary path:  {}", path.display()),
            Err(_) => println!("Binary path:  (not found in PATH)"),
        }
    }

    Ok(())
}

fn cmd_tool(
    sub: &ToolCommand,
    cli: &Cli,
    config: Option<&ubt_cli::config::UbtConfig>,
    project_root: &std::path::Path,
    registry: &PluginRegistry,
) -> Result<(), UbtError> {
    match sub {
        ToolCommand::Info => cmd_info(cli, config, project_root, registry),
        ToolCommand::Doctor => {
            let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
            match detect_tool(cli.tool.as_deref(), config_tool, project_root, registry) {
                Ok(detection) => {
                    if let Some((plugin, _)) = registry.get(&detection.plugin_name) {
                        if let Some(variant) = plugin.variants.get(&detection.variant_name) {
                            match which::which(&variant.binary) {
                                Ok(path) => {
                                    println!(
                                        "{} {} is installed at {}",
                                        detection.plugin_name,
                                        variant.binary,
                                        path.display()
                                    );
                                }
                                Err(_) => {
                                    eprintln!("{} is not installed.", variant.binary);
                                    if let Some(help) = &plugin.install_help {
                                        eprintln!("Install: {help}");
                                    }
                                    process::exit(1);
                                }
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{e}");
                    Ok(())
                }
            }
        }
        ToolCommand::List => {
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
            Ok(())
        }
        ToolCommand::Docs => {
            let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
            let detection = detect_tool(cli.tool.as_deref(), config_tool, project_root, registry)?;
            if let Some((plugin, _)) = registry.get(&detection.plugin_name) {
                if let Some(hp) = &plugin.homepage {
                    println!("{hp}");
                } else {
                    println!("No documentation URL configured for {}", plugin.name);
                }
            }
            Ok(())
        }
    }
}

fn cmd_config_show() -> Result<(), UbtError> {
    let cwd = std::env::current_dir()?;
    match load_config(&cwd)? {
        Some((config, root)) => {
            println!("Config file: {}", root.join("ubt.toml").display());
            if let Some(project) = &config.project {
                if let Some(tool) = &project.tool {
                    println!("Tool: {tool}");
                }
            }
            if !config.commands.is_empty() {
                println!("\nCommands:");
                let mut keys: Vec<_> = config.commands.keys().collect();
                keys.sort();
                for key in keys {
                    println!("  {key} = {:?}", config.commands[key]);
                }
            }
            if !config.aliases.is_empty() {
                println!("\nAliases:");
                let mut keys: Vec<_> = config.aliases.keys().collect();
                keys.sort();
                for key in keys {
                    println!("  {key} = {:?}", config.aliases[key]);
                }
            }
            Ok(())
        }
        None => {
            println!("No ubt.toml found.");
            Ok(())
        }
    }
}

fn cmd_init() -> Result<(), UbtError> {
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
        Err(_) => (
            "npm".to_string(),
            r#"start = "npm run dev""#.to_string(),
        ),
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

fn cmd_alias(
    ext_args: &[String],
    config: Option<&ubt_cli::config::UbtConfig>,
) -> Result<(), UbtError> {
    let alias_name = &ext_args[0];
    let remaining = &ext_args[1..];

    let cmd_str = config
        .and_then(|cfg| cfg.aliases.get(alias_name))
        .ok_or_else(|| UbtError::UnknownCommand {
            name: alias_name.clone(),
        })?;

    let args_str = remaining.join(" ");
    let expanded = if cmd_str.contains("{{args}}") {
        cmd_str.replace("{{args}}", &args_str)
    } else if remaining.is_empty() {
        cmd_str.clone()
    } else {
        format!("{} {}", cmd_str, args_str)
    };

    let exit_code = spawn_command(&expanded, None)?;
    process::exit(exit_code);
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
