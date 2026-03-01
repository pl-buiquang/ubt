use clap::{Parser, Subcommand};

// ── Top-level CLI ──────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "ubt", version, about = "Universal Build Tool")]
pub struct Cli {
    /// Enable verbose/debug output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Force a specific tool/runtime
    #[arg(long, global = true, env = "UBT_TOOL")]
    pub tool: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

// ── Top-level command enum ─────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Dependency management
    #[command(subcommand)]
    Dep(DepCommand),

    /// Build the project
    Build(BuildArgs),

    /// Start the project (dev server, etc.)
    Start(PassthroughArgs),

    /// Run a project script
    Run(RunArgs),

    /// Format source code
    Fmt(FmtArgs),

    /// Run a file directly
    #[command(name = "run-file", alias = "run:file")]
    RunFile(RunFileArgs),

    /// Execute an arbitrary command via the tool
    Exec(ExecArgs),

    /// Run tests
    Test(TestArgs),

    /// Lint source code
    Lint(LintArgs),

    /// Type-check / compile-check without producing output
    Check(PassthroughArgs),

    /// Database operations
    #[command(subcommand)]
    Db(DbCommand),

    /// Initialize a new project configuration
    Init,

    /// Clean build artifacts
    Clean(PassthroughArgs),

    /// Create a release
    Release(ReleaseArgs),

    /// Publish a package
    Publish(PublishArgs),

    /// Tool information and diagnostics
    #[command(subcommand)]
    Tool(ToolCommand),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Show detected tool/runtime info
    Info,

    /// Generate shell completions
    Completions(CompletionsArgs),

    /// Catch-all for alias dispatch from [aliases] in ubt.toml
    #[command(external_subcommand)]
    External(Vec<String>),
}

// ── Shared passthrough args ────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct PassthroughArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Dep subcommands ────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum DepCommand {
    /// Install dependencies
    Install(PassthroughArgs),

    /// Remove a dependency
    Remove(PassthroughArgs),

    /// Update dependencies
    Update(PassthroughArgs),

    /// Show outdated dependencies
    Outdated(PassthroughArgs),

    /// List installed dependencies
    List(PassthroughArgs),

    /// Audit dependencies for vulnerabilities
    Audit(PassthroughArgs),

    /// Generate or update lock file
    Lock(PassthroughArgs),

    /// Explain why a dependency is installed
    Why(PassthroughArgs),
}

// ── Build args ─────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct BuildArgs {
    /// Development build
    #[arg(long)]
    pub dev: bool,

    /// Watch mode — rebuild on changes
    #[arg(long)]
    pub watch: bool,

    /// Clean before building
    #[arg(long)]
    pub clean: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Run args ───────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct RunArgs {
    /// Script name to run
    pub script: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Fmt args ───────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct FmtArgs {
    /// Check formatting without modifying files
    #[arg(long)]
    pub check: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── RunFile args ───────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct RunFileArgs {
    /// File to run
    pub file: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Exec args ──────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct ExecArgs {
    /// Command to execute
    pub cmd: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Test args ──────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct TestArgs {
    /// Watch mode — rerun tests on changes
    #[arg(long)]
    pub watch: bool,

    /// Collect coverage information
    #[arg(long)]
    pub coverage: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Lint args ──────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct LintArgs {
    /// Automatically fix lint issues
    #[arg(long)]
    pub fix: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Db subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum DbCommand {
    /// Run database migrations
    Migrate(PassthroughArgs),

    /// Rollback database migrations
    Rollback(PassthroughArgs),

    /// Seed the database
    Seed(PassthroughArgs),

    /// Create the database
    Create(PassthroughArgs),

    /// Drop the database
    Drop(DbDropArgs),

    /// Reset the database (drop + create + migrate)
    Reset(DbResetArgs),

    /// Show migration status
    Status(PassthroughArgs),
}

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct DbDropArgs {
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct DbResetArgs {
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Release args ───────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct ReleaseArgs {
    /// Perform a dry run without making changes
    #[arg(long)]
    pub dry_run: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Publish args ───────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct PublishArgs {
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,

    /// Perform a dry run without publishing
    #[arg(long)]
    pub dry_run: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

// ── Tool subcommands ───────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ToolCommand {
    /// Show detected tool information
    Info,

    /// Run diagnostic checks
    Doctor,

    /// List available tools/plugins
    List,

    /// Open tool documentation
    Docs(DocsArgs),
}

#[derive(Parser, Debug)]
pub struct DocsArgs {
    /// Open the documentation URL in the system browser
    #[arg(long)]
    pub open: bool,
}

// ── Config subcommands ─────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
}

// ── Completions args ───────────────────────────────────────────────────

#[derive(Parser, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: clap_complete::Shell,
}

// ── Universal flags ────────────────────────────────────────────────────

#[derive(Debug, Default, PartialEq)]
pub struct UniversalFlags {
    pub watch: bool,
    pub coverage: bool,
    pub dev: bool,
    pub clean: bool,
    pub fix: bool,
    pub check: bool,
    pub yes: bool,
    pub dry_run: bool,
}

// ── Helper functions ───────────────────────────────────────────────────

/// Returns the dot-notation command name and an optional reference to the
/// passthrough args vec for the given command variant.
pub fn command_parts(cmd: &Command) -> (&'static str, Option<&Vec<String>>) {
    match cmd {
        Command::Dep(sub) => match sub {
            DepCommand::Install(a) => ("dep.install", Some(&a.args)),
            DepCommand::Remove(a) => ("dep.remove", Some(&a.args)),
            DepCommand::Update(a) => ("dep.update", Some(&a.args)),
            DepCommand::Outdated(a) => ("dep.outdated", Some(&a.args)),
            DepCommand::List(a) => ("dep.list", Some(&a.args)),
            DepCommand::Audit(a) => ("dep.audit", Some(&a.args)),
            DepCommand::Lock(a) => ("dep.lock", Some(&a.args)),
            DepCommand::Why(a) => ("dep.why", Some(&a.args)),
        },
        Command::Build(a) => ("build", Some(&a.args)),
        Command::Start(a) => ("start", Some(&a.args)),
        Command::Run(a) => ("run", Some(&a.args)),
        Command::Fmt(a) => ("fmt", Some(&a.args)),
        Command::RunFile(a) => ("run-file", Some(&a.args)),
        Command::Exec(a) => ("exec", Some(&a.args)),
        Command::Test(a) => ("test", Some(&a.args)),
        Command::Lint(a) => ("lint", Some(&a.args)),
        Command::Check(a) => ("check", Some(&a.args)),
        Command::Db(sub) => match sub {
            DbCommand::Migrate(a) => ("db.migrate", Some(&a.args)),
            DbCommand::Rollback(a) => ("db.rollback", Some(&a.args)),
            DbCommand::Seed(a) => ("db.seed", Some(&a.args)),
            DbCommand::Create(a) => ("db.create", Some(&a.args)),
            DbCommand::Drop(a) => ("db.drop", Some(&a.args)),
            DbCommand::Reset(a) => ("db.reset", Some(&a.args)),
            DbCommand::Status(a) => ("db.status", Some(&a.args)),
        },
        Command::Init => ("init", None),
        Command::Clean(a) => ("clean", Some(&a.args)),
        Command::Release(a) => ("release", Some(&a.args)),
        Command::Publish(a) => ("publish", Some(&a.args)),
        Command::Tool(sub) => match sub {
            ToolCommand::Info => ("tool.info", None),
            ToolCommand::Doctor => ("tool.doctor", None),
            ToolCommand::List => ("tool.list", None),
            ToolCommand::Docs(_) => ("tool.docs", None),
        },
        Command::Config(sub) => match sub {
            ConfigCommand::Show => ("config.show", None),
        },
        Command::Info => ("info", None),
        Command::Completions(..) => ("completions", None),
        Command::External(..) => unreachable!("External is dispatched before command_parts"),
    }
}

/// Returns a dot-notation name for the given command variant.
pub fn parse_command_name(cmd: &Command) -> &'static str {
    command_parts(cmd).0
}

/// Extracts known universal flags from a command variant.
pub fn collect_universal_flags(cmd: &Command) -> UniversalFlags {
    match cmd {
        Command::Build(args) => UniversalFlags {
            dev: args.dev,
            watch: args.watch,
            clean: args.clean,
            ..Default::default()
        },
        Command::Test(args) => UniversalFlags {
            watch: args.watch,
            coverage: args.coverage,
            ..Default::default()
        },
        Command::Lint(args) => UniversalFlags {
            fix: args.fix,
            ..Default::default()
        },
        Command::Fmt(args) => UniversalFlags {
            check: args.check,
            ..Default::default()
        },
        Command::Db(DbCommand::Drop(args)) => UniversalFlags {
            yes: args.yes,
            ..Default::default()
        },
        Command::Db(DbCommand::Reset(args)) => UniversalFlags {
            yes: args.yes,
            ..Default::default()
        },
        Command::Release(args) => UniversalFlags {
            dry_run: args.dry_run,
            ..Default::default()
        },
        Command::Publish(args) => UniversalFlags {
            yes: args.yes,
            dry_run: args.dry_run,
            ..Default::default()
        },
        Command::External(..) => {
            unreachable!("External is dispatched before collect_universal_flags")
        }
        _ => UniversalFlags::default(),
    }
}

/// Collects the passthrough/trailing args from any command variant.
pub fn collect_remaining_args(cmd: &Command) -> Vec<String> {
    command_parts(cmd).1.cloned().unwrap_or_default()
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // Helper to parse CLI from arguments
    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(args)
    }

    // ── Command name mapping ───────────────────────────────────────────

    #[test]
    fn command_name_dep_install() {
        let cli = parse(&["ubt", "dep", "install"]);
        assert_eq!(parse_command_name(&cli.command), "dep.install");
    }

    #[test]
    fn command_name_dep_remove() {
        let cli = parse(&["ubt", "dep", "remove"]);
        assert_eq!(parse_command_name(&cli.command), "dep.remove");
    }

    #[test]
    fn command_name_dep_update() {
        let cli = parse(&["ubt", "dep", "update"]);
        assert_eq!(parse_command_name(&cli.command), "dep.update");
    }

    #[test]
    fn command_name_dep_outdated() {
        let cli = parse(&["ubt", "dep", "outdated"]);
        assert_eq!(parse_command_name(&cli.command), "dep.outdated");
    }

    #[test]
    fn command_name_dep_list() {
        let cli = parse(&["ubt", "dep", "list"]);
        assert_eq!(parse_command_name(&cli.command), "dep.list");
    }

    #[test]
    fn command_name_dep_audit() {
        let cli = parse(&["ubt", "dep", "audit"]);
        assert_eq!(parse_command_name(&cli.command), "dep.audit");
    }

    #[test]
    fn command_name_dep_lock() {
        let cli = parse(&["ubt", "dep", "lock"]);
        assert_eq!(parse_command_name(&cli.command), "dep.lock");
    }

    #[test]
    fn command_name_dep_why() {
        let cli = parse(&["ubt", "dep", "why"]);
        assert_eq!(parse_command_name(&cli.command), "dep.why");
    }

    #[test]
    fn command_name_build() {
        let cli = parse(&["ubt", "build"]);
        assert_eq!(parse_command_name(&cli.command), "build");
    }

    #[test]
    fn command_name_start() {
        let cli = parse(&["ubt", "start"]);
        assert_eq!(parse_command_name(&cli.command), "start");
    }

    #[test]
    fn command_name_run() {
        let cli = parse(&["ubt", "run", "dev"]);
        assert_eq!(parse_command_name(&cli.command), "run");
    }

    #[test]
    fn command_name_fmt() {
        let cli = parse(&["ubt", "fmt"]);
        assert_eq!(parse_command_name(&cli.command), "fmt");
    }

    #[test]
    fn command_name_run_file() {
        let cli = parse(&["ubt", "run-file", "script.ts"]);
        assert_eq!(parse_command_name(&cli.command), "run-file");
    }

    #[test]
    fn command_name_exec() {
        let cli = parse(&["ubt", "exec", "node"]);
        assert_eq!(parse_command_name(&cli.command), "exec");
    }

    #[test]
    fn command_name_test() {
        let cli = parse(&["ubt", "test"]);
        assert_eq!(parse_command_name(&cli.command), "test");
    }

    #[test]
    fn command_name_lint() {
        let cli = parse(&["ubt", "lint"]);
        assert_eq!(parse_command_name(&cli.command), "lint");
    }

    #[test]
    fn command_name_check() {
        let cli = parse(&["ubt", "check"]);
        assert_eq!(parse_command_name(&cli.command), "check");
    }

    #[test]
    fn command_name_db_migrate() {
        let cli = parse(&["ubt", "db", "migrate"]);
        assert_eq!(parse_command_name(&cli.command), "db.migrate");
    }

    #[test]
    fn command_name_db_rollback() {
        let cli = parse(&["ubt", "db", "rollback"]);
        assert_eq!(parse_command_name(&cli.command), "db.rollback");
    }

    #[test]
    fn command_name_db_seed() {
        let cli = parse(&["ubt", "db", "seed"]);
        assert_eq!(parse_command_name(&cli.command), "db.seed");
    }

    #[test]
    fn command_name_db_create() {
        let cli = parse(&["ubt", "db", "create"]);
        assert_eq!(parse_command_name(&cli.command), "db.create");
    }

    #[test]
    fn command_name_db_drop() {
        let cli = parse(&["ubt", "db", "drop"]);
        assert_eq!(parse_command_name(&cli.command), "db.drop");
    }

    #[test]
    fn command_name_db_reset() {
        let cli = parse(&["ubt", "db", "reset"]);
        assert_eq!(parse_command_name(&cli.command), "db.reset");
    }

    #[test]
    fn command_name_db_status() {
        let cli = parse(&["ubt", "db", "status"]);
        assert_eq!(parse_command_name(&cli.command), "db.status");
    }

    #[test]
    fn command_name_init() {
        let cli = parse(&["ubt", "init"]);
        assert_eq!(parse_command_name(&cli.command), "init");
    }

    #[test]
    fn command_name_clean() {
        let cli = parse(&["ubt", "clean"]);
        assert_eq!(parse_command_name(&cli.command), "clean");
    }

    #[test]
    fn command_name_release() {
        let cli = parse(&["ubt", "release"]);
        assert_eq!(parse_command_name(&cli.command), "release");
    }

    #[test]
    fn command_name_publish() {
        let cli = parse(&["ubt", "publish"]);
        assert_eq!(parse_command_name(&cli.command), "publish");
    }

    #[test]
    fn command_name_tool_info() {
        let cli = parse(&["ubt", "tool", "info"]);
        assert_eq!(parse_command_name(&cli.command), "tool.info");
    }

    #[test]
    fn command_name_tool_doctor() {
        let cli = parse(&["ubt", "tool", "doctor"]);
        assert_eq!(parse_command_name(&cli.command), "tool.doctor");
    }

    #[test]
    fn command_name_tool_list() {
        let cli = parse(&["ubt", "tool", "list"]);
        assert_eq!(parse_command_name(&cli.command), "tool.list");
    }

    #[test]
    fn command_name_tool_docs() {
        let cli = parse(&["ubt", "tool", "docs"]);
        assert_eq!(parse_command_name(&cli.command), "tool.docs");
    }

    #[test]
    fn command_name_config_show() {
        let cli = parse(&["ubt", "config", "show"]);
        assert_eq!(parse_command_name(&cli.command), "config.show");
    }

    #[test]
    fn command_name_info() {
        let cli = parse(&["ubt", "info"]);
        assert_eq!(parse_command_name(&cli.command), "info");
    }

    #[test]
    fn command_name_completions() {
        let cli = parse(&["ubt", "completions", "bash"]);
        assert_eq!(parse_command_name(&cli.command), "completions");
    }

    // ── Global flag extraction ─────────────────────────────────────────

    #[test]
    fn global_verbose_flag() {
        let cli = parse(&["ubt", "-v", "info"]);
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn global_quiet_flag() {
        let cli = parse(&["ubt", "-q", "info"]);
        assert!(!cli.verbose);
        assert!(cli.quiet);
    }

    #[test]
    fn global_tool_flag() {
        let cli = parse(&["ubt", "--tool", "npm", "info"]);
        assert_eq!(cli.tool, Some("npm".to_string()));
    }

    #[test]
    fn global_tool_flag_absent() {
        let cli = parse(&["ubt", "info"]);
        assert_eq!(cli.tool, None);
    }

    // ── Universal flag extraction ──────────────────────────────────────

    #[test]
    fn universal_flags_build_dev() {
        let cli = parse(&["ubt", "build", "--dev"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.dev);
        assert!(!flags.watch);
        assert!(!flags.clean);
    }

    #[test]
    fn universal_flags_build_watch_clean() {
        let cli = parse(&["ubt", "build", "--watch", "--clean"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.watch);
        assert!(flags.clean);
    }

    #[test]
    fn universal_flags_test_coverage_watch() {
        let cli = parse(&["ubt", "test", "--coverage", "--watch"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.coverage);
        assert!(flags.watch);
    }

    #[test]
    fn universal_flags_lint_fix() {
        let cli = parse(&["ubt", "lint", "--fix"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.fix);
    }

    #[test]
    fn universal_flags_fmt_check() {
        let cli = parse(&["ubt", "fmt", "--check"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.check);
    }

    #[test]
    fn universal_flags_db_drop_yes() {
        let cli = parse(&["ubt", "db", "drop", "--yes"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.yes);
    }

    #[test]
    fn universal_flags_db_reset_yes() {
        let cli = parse(&["ubt", "db", "reset", "-y"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.yes);
    }

    #[test]
    fn universal_flags_publish_dry_run() {
        let cli = parse(&["ubt", "publish", "--dry-run"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.dry_run);
        assert!(!flags.yes);
    }

    #[test]
    fn universal_flags_publish_yes_and_dry_run() {
        let cli = parse(&["ubt", "publish", "--yes", "--dry-run"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.yes);
        assert!(flags.dry_run);
    }

    #[test]
    fn universal_flags_release_dry_run() {
        let cli = parse(&["ubt", "release", "--dry-run"]);
        let flags = collect_universal_flags(&cli.command);
        assert!(flags.dry_run);
    }

    #[test]
    fn universal_flags_default_for_info() {
        let cli = parse(&["ubt", "info"]);
        let flags = collect_universal_flags(&cli.command);
        assert_eq!(flags, UniversalFlags::default());
    }

    // ── Remaining args collection ──────────────────────────────────────

    #[test]
    fn remaining_args_build() {
        let cli = parse(&["ubt", "build", "--dev", "--", "extra1", "extra2"]);
        let args = collect_remaining_args(&cli.command);
        assert!(args.contains(&"extra1".to_string()));
        assert!(args.contains(&"extra2".to_string()));
    }

    #[test]
    fn remaining_args_start() {
        let cli = parse(&["ubt", "start", "foo", "bar"]);
        let args = collect_remaining_args(&cli.command);
        assert_eq!(args, vec!["foo", "bar"]);
    }

    #[test]
    fn remaining_args_test_with_flags() {
        let cli = parse(&["ubt", "test", "--watch", "some-pattern"]);
        let args = collect_remaining_args(&cli.command);
        assert_eq!(args, vec!["some-pattern"]);
    }

    #[test]
    fn remaining_args_dep_install() {
        let cli = parse(&["ubt", "dep", "install", "lodash", "express"]);
        let args = collect_remaining_args(&cli.command);
        assert_eq!(args, vec!["lodash", "express"]);
    }

    #[test]
    fn remaining_args_run() {
        let cli = parse(&["ubt", "run", "dev", "--port", "3000"]);
        let args = collect_remaining_args(&cli.command);
        assert_eq!(args, vec!["--port", "3000"]);
    }

    #[test]
    fn remaining_args_empty_for_init() {
        let cli = parse(&["ubt", "init"]);
        let args = collect_remaining_args(&cli.command);
        assert!(args.is_empty());
    }

    #[test]
    fn remaining_args_empty_for_info() {
        let cli = parse(&["ubt", "info"]);
        let args = collect_remaining_args(&cli.command);
        assert!(args.is_empty());
    }

    #[test]
    fn remaining_args_empty_for_tool() {
        let cli = parse(&["ubt", "tool", "info"]);
        let args = collect_remaining_args(&cli.command);
        assert!(args.is_empty());
    }

    #[test]
    fn remaining_args_exec() {
        let cli = parse(&["ubt", "exec", "node", "-e", "console.log(1)"]);
        let args = collect_remaining_args(&cli.command);
        assert_eq!(args, vec!["-e", "console.log(1)"]);
    }

    // ── Help & version output ──────────────────────────────────────────

    #[test]
    fn help_output_produces_error() {
        let result = Cli::try_parse_from(["ubt", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn version_output_produces_error() {
        let result = Cli::try_parse_from(["ubt", "--version"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    // ── Error on unknown command ───────────────────────────────────────

    #[test]
    fn unknown_command_routes_to_external() {
        let cli = parse(&["ubt", "nonexistent"]);
        assert!(matches!(cli.command, Command::External(ref args) if args[0] == "nonexistent"));
    }

    // ── Completions shell parsing ──────────────────────────────────────

    #[test]
    fn completions_bash() {
        let cli = parse(&["ubt", "completions", "bash"]);
        if let Command::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::Bash);
        } else {
            panic!("expected Completions command");
        }
    }

    #[test]
    fn completions_zsh() {
        let cli = parse(&["ubt", "completions", "zsh"]);
        if let Command::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::Zsh);
        } else {
            panic!("expected Completions command");
        }
    }

    #[test]
    fn completions_fish() {
        let cli = parse(&["ubt", "completions", "fish"]);
        if let Command::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::Fish);
        } else {
            panic!("expected Completions command");
        }
    }

    #[test]
    fn completions_powershell() {
        let cli = parse(&["ubt", "completions", "powershell"]);
        if let Command::Completions(args) = &cli.command {
            assert_eq!(args.shell, clap_complete::Shell::PowerShell);
        } else {
            panic!("expected Completions command");
        }
    }

    // ── run-file alias ─────────────────────────────────────────────────

    #[test]
    fn run_file_alias() {
        let cli = parse(&["ubt", "run:file", "script.ts"]);
        assert_eq!(parse_command_name(&cli.command), "run-file");
    }
}
