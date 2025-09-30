use std::process::ExitCode;

use clap::{ArgAction, Parser};
use zsh_history_cleaner::history;

/// Clean your commands history by removing duplicate commands, commands between dates, etc...
///
/// By default, all the duplicate commands are removed.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// Dry run mode. The history file is not modified when this flag is used.
    #[arg(short, long, action = ArgAction::SetTrue, default_value = "false")]
    dry_run: bool,

    /// The history file to use
    #[arg(short = 'H', long, default_value = "~/.zsh_history")]
    history_file: String,

    /// [USE WITH CAUTION!!] Disable history file backup. By default, a backup is written to '{history_file}.{timestamp}' in the current directory.
    #[arg(short, long, action = ArgAction::SetFalse)]
    no_backup: bool,

    /// Should we keep duplicate commands in the history file?
    #[arg(short, long, action = ArgAction::SetTrue, default_value = "false")]
    keep_duplicates: bool,

    /// Remove commands between two dates (YYYY-MM-DD YYYY-MM-DD)
    #[arg(short, long, num_args = 2, value_names = ["START_DATE", "END_DATE"], value_parser = validate_date)]
    remove_between: Option<Vec<String>>,
}

/// Validate that the date string is in the format YYYY-MM-DD
fn validate_date(date_str: &str) -> Result<String, String> {
    if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
        return Err(format!(
            "Invalid date format for '{}'. Expected format: YYYY-MM-DD",
            date_str
        ));
    }
    Ok(date_str.to_string())
}

fn run(cli: Cli) -> Result<Option<String>, String> {
    // TODO: Check if we can move this in the CLI parser instead
    if let Some(dates) = cli.remove_between {
        let start_date = &dates[0];
        let end_date = &dates[1];

        if start_date > end_date {
            return Err(format!(
                "Start date '{}' is after end date '{}'. Please provide valid dates.",
                start_date, end_date
            ));
        }
    }

    let mut history =
        history::History::from_file(&cli.history_file).map_err(|err| err.to_string())?;
    let backup_flag = cli.no_backup;

    if history.is_empty() {
        println!(
            "No entries found in the history file '{}' Nothing to do.",
            history.filename()
        );
        return Ok(None);
    }

    let initial_size = history.size();

    println!("{} entries in '{}'", history.size(), history.filename());

    if !cli.keep_duplicates {
        history.remove_duplicates();
    }

    if history.size() == initial_size {
        println!("No changes made to the history file.");
        return Ok(None);
    }

    if cli.dry_run {
        println!("Dry run enabled: No changes were saved.");
        return Ok(None);
    }

    history.write(backup_flag).map_err(|err| err.to_string())
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
