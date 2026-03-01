use std::process;

use crate::config::UbtConfig;
use crate::error::UbtError;
use crate::executor::spawn_command;

pub fn cmd_alias(ext_args: &[String], config: Option<&UbtConfig>) -> Result<(), UbtError> {
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
