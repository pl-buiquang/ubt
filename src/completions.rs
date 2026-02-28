use clap::CommandFactory;
use clap_complete::Shell;
use std::io::Write;

use crate::cli::Cli;

/// Generate shell completions for the given shell and write to the provided writer.
pub fn generate_completions(shell: Shell, writer: &mut dyn Write) {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "ubt", writer);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completions_for(shell: Shell) -> String {
        let mut buf = Vec::new();
        generate_completions(shell, &mut buf);
        String::from_utf8(buf).expect("completions should be valid UTF-8")
    }

    #[test]
    fn bash_completions_non_empty() {
        let output = completions_for(Shell::Bash);
        assert!(!output.is_empty());
    }

    #[test]
    fn bash_completions_has_marker() {
        let output = completions_for(Shell::Bash);
        assert!(output.contains("complete"));
    }

    #[test]
    fn zsh_completions_non_empty() {
        let output = completions_for(Shell::Zsh);
        assert!(!output.is_empty());
    }

    #[test]
    fn zsh_completions_has_compdef() {
        let output = completions_for(Shell::Zsh);
        assert!(output.contains("#compdef"));
    }

    #[test]
    fn fish_completions_non_empty() {
        let output = completions_for(Shell::Fish);
        assert!(!output.is_empty());
    }

    #[test]
    fn powershell_completions_non_empty() {
        let output = completions_for(Shell::PowerShell);
        assert!(!output.is_empty());
    }
}
