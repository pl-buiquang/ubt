use crate::cli::Cli;
use crate::config::UbtConfig;
use crate::detect::detect_tool;
use crate::error::UbtError;
use crate::plugin::PluginRegistry;

pub fn cmd_info(
    cli: &Cli,
    config: Option<&UbtConfig>,
    project_root: &std::path::Path,
    registry: &PluginRegistry,
) -> Result<(), UbtError> {
    let config_tool = config.and_then(|c| c.project.as_ref()?.tool.as_deref());
    let detection = detect_tool(cli.tool.as_deref(), config_tool, project_root, registry)?;

    if cli.verbose && !cli.quiet {
        eprintln!(
            "ubt: detected {} (variant: {}) at {}",
            detection.plugin_name,
            detection.variant_name,
            detection.project_root.display()
        );
    }

    if cli.quiet {
        return Ok(());
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
