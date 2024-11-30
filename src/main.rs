use std::process::ExitCode;

use clap::{ArgAction, Parser};
use zsh_history_cleaner::history;

/// Clean your history by removing duplicate commands, commands matching regex etc...
///
/// By default, all the duplicate commands are removed.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Dry run mode. The history file is not modified when this flag is used.
    #[arg(short, long, action = ArgAction::SetTrue, default_value = "false")]
    dry_run: bool,

    /// The history file to use.
    #[arg(short = 'H', long, default_value = "~/.zsh_history")]
    history_file: String,

    /// [USE WITH CAUTION!!] Disable history file backup. By default, a backup is written to '{history_file}.{timestamp}' in the current directory.
    #[arg(short, long, action = ArgAction::SetFalse, default_value = "false")]
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

    if history.is_empty() {
        println!(
            "No entries found in the history file '{}'.",
            cli.history_file
        );
        return Ok(());
    }

    println!("{} entries in '{}'", history.size(), cli.history_file);

    history.remove_duplicates();

    if cli.dry_run {
        println!("Dry run enabled. No changes will be made.");
        return Ok(());
    }

    history.write(cli.no_backup).map_err(|err| err.to_string())
}
