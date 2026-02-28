use clap::Parser;
use ubt::cli::Cli;

fn main() {
    let cli = Cli::parse();
    if cli.verbose {
        eprintln!("debug: parsed command = {:?}", cli.command);
    }
}
