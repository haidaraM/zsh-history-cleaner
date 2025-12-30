use std::process::ExitCode;

use chrono::NaiveDate;
use clap::{ArgAction, Parser};
use zsh_history_cleaner::history;
use zsh_history_cleaner::utils::TERMINAL_MAX_WIDTH;

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

    /// [USE WITH CAUTION!!] Disable the history file backup. By default, a backup is written to '{history_file_path}.zhc_backup_{timestamp}'.
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    no_backup: bool,

    /// Should we keep duplicate commands in the history file?
    #[arg(short, long, action = ArgAction::SetTrue, default_value = "false")]
    keep_duplicates: bool,

    /// Remove commands between the provided two dates (included): YYYY-MM-DD YYYY-MM-DD. The first date must be before or equal to the second date.
    /// Example: --remove-between 2023-01-01 2023-06-30
    #[arg(short, long, num_args = 2, value_names = ["START_DATE", "END_DATE"], value_parser = validate_date)]
    remove_between: Option<Vec<NaiveDate>>,

    /// Analyze the history file and provide statistics about the commands over time.
    /// No changes are made to the history file when this flag is used.
    #[arg(short, long)]
    analyze: bool,
}

impl Cli {
    /// Validates that the date range is valid (start <= end)
    /// Call this after parsing to ensure business logic constraints
    fn validate(&self) -> Result<(), String> {
        if let Some(dates) = &self.remove_between
            && dates[0] > dates[1]
        {
            return Err(format!(
                "Start date '{}' is after end date '{}'. Please provide valid dates.",
                dates[0], dates[1]
            ));
        }
        Ok(())
    }
}

/// Validate that the date string is in the format YYYY-MM-DD
fn validate_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
        format!(
            "Invalid date format for '{}'. Expected format: YYYY-MM-DD",
            date_str
        )
    })
}

fn run(cli: Cli) -> Result<Option<String>, String> {
    let mut history =
        history::History::from_file(&cli.history_file).map_err(|err| err.to_string())?;

    let should_backup = !cli.no_backup;

    if history.is_empty() {
        println!(
            "No entries found in the history file '{}'.",
            history.filename()
        );
        return Ok(None);
    }

    if cli.dry_run && !cli.analyze {
        println!("{}", "━".repeat(TERMINAL_MAX_WIDTH.into()));
        println!("Dry run mode enabled: no changes will be saved to the filesystem.");
        println!("{}", "━".repeat(TERMINAL_MAX_WIDTH.into()));
    }

    if cli.analyze {
        let time_analysis = history.analyze_by_time();
        println!("{}", time_analysis);
        return Ok(None);
    }

    let initial_size = history.size();

    println!("{} entries in '{}'", history.size(), history.filename());

    if !cli.keep_duplicates {
        let count = history.remove_duplicates();
        println!("{} duplicate commands found.", count);
    }

    if let Some(dates) = cli.remove_between {
        history.remove_between_dates(&dates[0], &dates[1]);
    }

    if history.size() == initial_size {
        println!("No changes were made to the history file.");
        return Ok(None);
    }

    if !cli.dry_run {
        println!("Saving changes to the filesystem.");
        history.write(should_backup).map_err(|err| err.to_string())
    } else {
        Ok(None)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Validate "business logic constraints" after parsing
    if let Err(err) = cli.validate() {
        eprintln!("Error: {}", err);
        return ExitCode::FAILURE;
    }

    if let Err(err) = run(cli) {
        eprintln!("Error: {}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
