use std::process;

use crate::cli::{Cli, ToolCommand};
use crate::config::UbtConfig;
use crate::detect::detect_tool;
use crate::error::UbtError;
use crate::plugin::PluginRegistry;

use super::info::cmd_info;

pub fn cmd_tool(
    sub: &ToolCommand,
    cli: &Cli,
    config: Option<&UbtConfig>,
    project_root: &std::path::Path,
    registry: &PluginRegistry,
) -> Result<(), UbtError> {
    match sub {
        ToolCommand::Info => cmd_info(cli, config, project_root, registry),
        ToolCommand::Doctor => {
            let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
            match detect_tool(cli.tool.as_deref(), config_tool, project_root, registry) {
                Ok(detection) => {
                    if let Some((plugin, _)) = registry.get(&detection.plugin_name)
                        && let Some(variant) = plugin.variants.get(&detection.variant_name)
                    {
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
        ToolCommand::Docs(args) => {
            let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
            let detection = detect_tool(cli.tool.as_deref(), config_tool, project_root, registry)?;
            if let Some((plugin, _)) = registry.get(&detection.plugin_name) {
                if let Some(hp) = &plugin.homepage {
                    if args.open {
                        if let Err(e) = open::that(hp) {
                            eprintln!("ubt: could not open browser: {e}");
                            println!("{hp}");
                        }
                    } else {
                        println!("{hp}");
                    }
                } else {
                    println!("No documentation URL configured for {}", plugin.name);
                }
            }
            Ok(())
        }
    }
}
