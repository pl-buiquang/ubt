use crate::config::load_config;
use crate::error::UbtError;

pub fn cmd_config_show() -> Result<(), UbtError> {
    let cwd = std::env::current_dir()?;
    match load_config(&cwd)? {
        Some((config, root)) => {
            println!("Config file: {}", root.join("ubt.toml").display());
            if let Some(project) = &config.project
                && let Some(tool) = &project.tool
            {
                println!("Tool: {tool}");
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
