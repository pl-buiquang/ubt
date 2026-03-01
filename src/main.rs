use std::process;

use clap::Parser;
use ubt_cli::cli::{
    Cli, Command, ConfigCommand, RunArgs, RunFileArgs, collect_remaining_args,
    collect_universal_flags, parse_command_name,
};
use ubt_cli::commands::{cmd_alias, cmd_config_show, cmd_info, cmd_init, cmd_tool};
use ubt_cli::completions::generate_completions;
use ubt_cli::config::load_config;
use ubt_cli::detect::detect_tool;
use ubt_cli::error::UbtError;
use ubt_cli::executor::{ResolveContext, execute_command, resolve_command};
use ubt_cli::plugin::PluginRegistry;

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

    if cli.verbose && !cli.quiet {
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

    if cli.verbose && !cli.quiet {
        eprintln!("ubt: executing: {cmd_str}");
    }

    // Execute (replaces current process on Unix via exec())
    let exit_code = execute_command(&cmd_str, resolved.install_help.as_deref())?;
    process::exit(exit_code);
}
