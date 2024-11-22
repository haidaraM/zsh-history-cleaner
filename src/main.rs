use std::process::ExitCode;

use clap::{ArgAction, Parser};
use zsh_history_cleaner::history;

/// Clean your history by removing duplicate commands, commands matching regex etc...
///
/// By default, all the duplicate commands are removed.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Dry run mode. The history file is not modified.
    #[arg(short, long, action = ArgAction::SetTrue, default_value = "false")]
    dry_run: bool,

    /// History file path.
    #[arg(short = 'H', long, default_value = "~/.zsh_history")]
    history_file: String,

    /// Disable history file backup. By default, a backup is written to {history_file}.{timestamp} in the current directory. Use with caution!!
    #[arg(short, long, action = ArgAction::SetFalse)]
    no_backup: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("{}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let mut history =
        history::History::from_file(&cli.history_file).map_err(|err| err.to_string())?;

    if history.entries.is_empty() {
        println!("No entries found in the history file '{}'.", cli.history_file);
        return Ok(());
    }

    println!(
        "{} entries in '{}'",
        history.entries.len(),
        cli.history_file
    );

    if cli.dry_run {
        println!("Dry run enabled. No changes will be made.");
        return Ok(());
    }

    history.remove_duplicates();

    history.write(cli.no_backup).map_err(|err| err.to_string())
}
