mod errors;
mod history;

use clap::{ArgAction, Parser};

/// Clean your history by removing duplicate commands, commands matching regex etc...
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Dry run mode. The history file is not modified.
    #[arg(short, long, action = ArgAction::SetTrue, default_value = "false")]
    dry_run: bool,

    /// History file path.
    #[arg(short = 'H', long, default_value = "~/.zsh_history")]
    history_file: String,

    /// Disable history file backup. By default, a backup is written to {history_file}.{timestamp}. Use with caution!!
    #[arg(short, long, action = ArgAction::SetFalse)]
    no_backup: bool,
}
fn main() {
    let cli = Cli::parse();
    println!("{:#?}", cli);

    if cli.dry_run {
        println!("Running in dry-run mode");
    }
}
