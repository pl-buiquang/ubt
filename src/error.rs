use thiserror::Error;

/// Unified error type for UBT.
#[derive(Debug, Error)]
pub enum UbtError {
    #[error("Error: {tool} is not installed.{install_guidance}")]
    ToolNotFound {
        tool: String,
        install_guidance: String,
    },

    #[error("\"{command}\" is not supported by the {plugin} plugin. {hint}")]
    CommandUnsupported {
        command: String,
        plugin: String,
        hint: String,
    },

    #[error("No command configured for \"{command}\". Add it to ubt.toml:\n\n  [commands]\n  \"{command}\" = \"your command here\"")]
    CommandUnmapped { command: String },

    #[error("{message}")]
    ConfigError { message: String },

    #[error("Multiple plugins detected: {plugins}. Set tool in ubt.toml:\n\n  [project]\n  tool = \"{suggested_tool}\"")]
    PluginConflict {
        plugins: String,
        suggested_tool: String,
    },

    #[error("Could not detect project type. Run \"ubt init\" or create ubt.toml.")]
    NoPluginMatch,

    #[error("Failed to load plugin \"{name}\": {detail}")]
    PluginLoadError { name: String, detail: String },

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Alias \"{alias}\" conflicts with built-in command \"{command}\"")]
    AliasConflict { alias: String, command: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience result type for UBT operations.
pub type Result<T> = std::result::Result<T, UbtError>;

impl UbtError {
    /// Create a `ToolNotFound` error with optional install guidance.
    pub fn tool_not_found(tool: impl Into<String>, install_help: Option<&str>) -> Self {
        let install_guidance = match install_help {
            Some(help) => format!("\n\n{help}"),
            None => String::new(),
        };
        UbtError::ToolNotFound {
            tool: tool.into(),
            install_guidance,
        }
    }

    /// Create a `ConfigError` with optional line number context.
    pub fn config_error(line: Option<usize>, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        let message = match line {
            Some(n) => format!("Error in ubt.toml [line {n}]: {detail}"),
            None => format!("Error in ubt.toml: {detail}"),
        };
        UbtError::ConfigError { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_not_found_with_install_help() {
        let err = UbtError::tool_not_found("npm", Some("Install via: https://nodejs.org"));
        assert_eq!(
            err.to_string(),
            "Error: npm is not installed.\n\nInstall via: https://nodejs.org"
        );
    }

    #[test]
    fn tool_not_found_without_install_help() {
        let err = UbtError::tool_not_found("cargo", None);
        assert_eq!(err.to_string(), "Error: cargo is not installed.");
    }

    #[test]
    fn command_unsupported_formats() {
        let err = UbtError::CommandUnsupported {
            command: "lint".into(),
            plugin: "go".into(),
            hint: "Try \"ubt run lint\" instead.".into(),
        };
        assert_eq!(
            err.to_string(),
            "\"lint\" is not supported by the go plugin. Try \"ubt run lint\" instead."
        );
    }

    #[test]
    fn command_unmapped_formats() {
        let err = UbtError::CommandUnmapped {
            command: "deploy".into(),
        };
        assert_eq!(
            err.to_string(),
            "No command configured for \"deploy\". Add it to ubt.toml:\n\n  [commands]\n  \"deploy\" = \"your command here\""
        );
    }

    #[test]
    fn config_error_with_line() {
        let err = UbtError::config_error(Some(42), "invalid key");
        assert_eq!(
            err.to_string(),
            "Error in ubt.toml [line 42]: invalid key"
        );
    }

    #[test]
    fn config_error_without_line() {
        let err = UbtError::config_error(None, "missing section");
        assert_eq!(err.to_string(), "Error in ubt.toml: missing section");
    }

    #[test]
    fn plugin_conflict_formats() {
        let err = UbtError::PluginConflict {
            plugins: "node, bun".into(),
            suggested_tool: "node".into(),
        };
        assert_eq!(
            err.to_string(),
            "Multiple plugins detected: node, bun. Set tool in ubt.toml:\n\n  [project]\n  tool = \"node\""
        );
    }

    #[test]
    fn no_plugin_match_formats() {
        let err = UbtError::NoPluginMatch;
        assert_eq!(
            err.to_string(),
            "Could not detect project type. Run \"ubt init\" or create ubt.toml."
        );
    }

    #[test]
    fn plugin_load_error_formats() {
        let err = UbtError::PluginLoadError {
            name: "rust".into(),
            detail: "file not found".into(),
        };
        assert_eq!(
            err.to_string(),
            "Failed to load plugin \"rust\": file not found"
        );
    }

    #[test]
    fn template_error_formats() {
        let err = UbtError::TemplateError("unresolved placeholder".into());
        assert_eq!(err.to_string(), "Template error: unresolved placeholder");
    }

    #[test]
    fn execution_error_formats() {
        let err = UbtError::ExecutionError("process killed".into());
        assert_eq!(err.to_string(), "Execution error: process killed");
    }

    #[test]
    fn alias_conflict_formats() {
        let err = UbtError::AliasConflict {
            alias: "t".into(),
            command: "test".into(),
        };
        assert_eq!(
            err.to_string(),
            "Alias \"t\" conflicts with built-in command \"test\""
        );
    }

    #[test]
    fn io_error_from_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let ubt_err: UbtError = io_err.into();
        assert!(matches!(ubt_err, UbtError::Io(_)));
        assert_eq!(ubt_err.to_string(), "gone");
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UbtError>();
    }

    #[test]
    fn result_type_alias_works() {
        fn returns_ok() -> Result<i32> {
            Ok(42)
        }
        fn returns_err() -> Result<i32> {
            Err(UbtError::NoPluginMatch)
        }
        assert_eq!(returns_ok().unwrap(), 42);
        assert!(returns_err().is_err());
    }
}
