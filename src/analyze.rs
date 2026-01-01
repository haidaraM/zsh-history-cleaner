//! Analysis utilities for Zsh history data.
//!
//! This module provides the [`HistoryAnalyzer`] service for analyzing
//! history entries and generating statistics.

use crate::history::History;
use crate::utils::{format_rank_icon, truncate_count_text, truncate_text_left};
use chrono::{Duration, Local, NaiveDate};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use console::{measure_text_width, style};
use humanize_duration::prelude::DurationExt;
use humanize_duration::Truncate;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Maximum width for terminal when displaying some things.
pub const TERMINAL_MAX_WIDTH: u8 = 90;

/// Maximum length for text in table cells
pub const MAX_CELL_TEXT_LENGTH: usize = 40;

/// Service for analyzing history data.
/// This struct provides various analysis capabilities on a History instance,
/// following the Service Pattern to separate analysis logic from data storage.
pub struct HistoryAnalyzer<'a> {
    history: &'a History,
}

impl<'a> HistoryAnalyzer<'a> {
    /// Creates a new HistoryAnalyzer for the given history.
    pub fn new(history: &'a History) -> Self {
        Self { history }
    }

    /// Analyze the History and return a HistoryAnalysis struct
    pub fn analyze(&self, top_n: usize) -> HistoryAnalysis {
        let date_range = self.date_range().unwrap_or_else(|| {
            let now = Local::now().date_naive();
            (now, now)
        });
        HistoryAnalysis {
            filename: self.history.filename().to_string(),
            size: self.history.size(),
            date_range,
            top_n,
            top_n_commands: self.top_n_commands(top_n),
            top_n_executables: self.top_n_executables(top_n),
            duplicate_counts: self.history.duplicate_commands_count(),
        }
    }

    /// Return the top n most frequent commands.
    /// If n is 0, returns an empty vector.
    pub fn top_n_commands(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 || self.history.is_empty() {
            return Vec::new();
        }

        // Count occurrences of each command. The key is the command string slice.
        let mut commands_count: HashMap<&str, usize> = HashMap::new();

        for entry in self.history.entries() {
            if let Some(command) = entry.valid_command() {
                *commands_count.entry(command).or_insert(0) += 1;
            }
        }

        let mut count_vec: Vec<(&str, usize)> = commands_count.into_iter().collect();
        // sort by count descending (then command name for ties), and take top n
        count_vec.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        count_vec.truncate(n);

        count_vec
            .into_iter()
            .map(|(cmd, count)| (cmd.to_string(), count))
            .collect()
    }

    /// Return the top n most frequent executables (first word of the command).
    /// If n is 0, returns an empty vector.
    pub fn top_n_executables(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 || self.history.is_empty() {
            return Vec::new();
        }

        // Count occurrences of each binary (first word of the command)
        let mut executables_count: HashMap<&str, usize> = HashMap::new();

        for entry in self.history.entries() {
            if let Some(command) = entry.valid_command()
                && let Some(binary) = command.split_whitespace().next()
            {
                *executables_count.entry(binary).or_insert(0) += 1;
            }
        }

        let mut count_vec: Vec<(&str, usize)> = executables_count.into_iter().collect();
        // sort by count descending (then binary name for ties), and take top n
        count_vec.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        count_vec.truncate(n);

        count_vec
            .into_iter()
            .map(|(bin, count)| (bin.to_string(), count))
            .collect()
    }

    /// Returns the range of dates covered by the commands (min_date, max_date)
    pub fn date_range(&self) -> Option<(NaiveDate, NaiveDate)> {
        self.history
            .entries()
            .iter()
            .filter_map(|entry| entry.timestamp_as_date_time())
            .map(|dt| dt.date_naive())
            .fold(None, |acc: Option<(NaiveDate, NaiveDate)>, current_date| {
                Some(match acc {
                    None => (current_date, current_date), // Initialize with the first date
                    Some((min, max)) => (min.min(current_date), max.max(current_date)),
                })
            })
    }
}

/// Represents the analysis of history commands by time
/// # Fields
/// - `filename`: The filename where the history was read
/// - `size`: The number of commands in the history
/// - `date_range`: The range of dates covered by the commands (min_date, max_date)
#[derive(Debug)]
pub struct HistoryAnalysis {
    /// The filename where the history was read
    pub filename: String,
    /// The number of commands in the history
    pub size: usize,
    /// The range of dates covered by the commands (min_date, max_date)
    pub date_range: (NaiveDate, NaiveDate),
    /// Duplicate counts
    pub duplicate_counts: usize,
    /// Top n
    pub top_n: usize,
    /// The top N most frequent commands
    pub top_n_commands: Vec<(String, usize)>,
    /// The top N most frequent executables
    pub top_n_executables: Vec<(String, usize)>,
    // The number of duplicate commands found
    // pub duplicates_count: usize,
    //pub commands_per_day: HashMap<NaiveDate, usize>,
    //pub commands_per_week: HashMap<u32, usize>, // Week number
    //pub commands_per_month: HashMap<(i32, u32), usize>, // (Year, Month)
    //pub commands_per_year: HashMap<i32, usize>, // Year
}

/// Display implementation for TimeAnalysis.
/// This formats the analysis in a human-readable way.
impl Display for HistoryAnalysis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let duration: Duration = self.date_range.1.signed_duration_since(self.date_range.0);
        let human_duration = duration.human(Truncate::Day);

        // Create a visually appealing stats box
        let box_width = TERMINAL_MAX_WIDTH as usize;
        let top_border = format!("╭{}╮", "─".repeat(box_width - 2));
        let bottom_border = format!("╰{}╯", "─".repeat(box_width - 2));

        // Helper closure to create a padded line for the box
        let make_box_line = |content: String| -> String {
            let visible_width = measure_text_width(&content);
            // Calculate padding: box_width - visible_width - 4 (for "│ " and " │")
            let padding_needed = box_width.saturating_sub(visible_width + 4);
            format!(
                "{} {}{} {}",
                style("│").blue(),
                content,
                " ".repeat(padding_needed),
                style("│").blue()
            )
        };

        // Format the title
        let box_title = format!(
            "📊 History Analysis for {}:",
            style(truncate_text_left(
                &self.filename,
                MAX_CELL_TEXT_LENGTH + 20
            ))
            .cyan()
            .bold()
        );

        // Format date range with colored dates
        let date_range_text = format!(
            "🗓️ {} → {} {}",
            style(&self.date_range.0).green().bold(),
            style(&self.date_range.1).green().bold(),
            style(format!("({})", human_duration)).dim().italic()
        );

        // Format total commands with highlighted number
        let total_commands = format!("📝 Total Commands: {}", style(&self.size).yellow().bold());
        let duplicate_commands_count = self.duplicate_counts;
        let duplicate_commands_percentage = if self.size > 0 {
            format!(
                "({:.2}%)",
                self.duplicate_counts as f64 / self.size as f64 * 100.0
            )
        } else {
            "(0.00%)".into()
        };
        let duplicate_commands = format!(
            "♻️ Duplicate Commands: {} {}",
            style(duplicate_commands_count).bold().yellow(),
            style(duplicate_commands_percentage).dim().italic()
        );

        // Print the stats box with properly aligned borders
        writeln!(f, "{}", style(top_border).blue())?;
        writeln!(f, "{}", make_box_line(box_title))?;
        writeln!(f, "{}", make_box_line("".into()))?;
        writeln!(f, "{}", make_box_line(date_range_text))?;
        writeln!(f, "{}", make_box_line(total_commands))?;
        writeln!(f, "{}", make_box_line(duplicate_commands))?;
        writeln!(f, "{}", style(bottom_border).blue())?;
        writeln!(f)?;

        // Section header for top items
        writeln!(
            f,
            "{} {}",
            style("🔥").bold(),
            style(format!("Top {} Most Used:", self.top_n))
                .magenta()
                .bold()
        )?;

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("").add_attribute(Attribute::Bold),
                Cell::new(style("Commands").cyan().bold().to_string())
                    .add_attribute(Attribute::Bold),
                Cell::new(style("Executables").cyan().bold().to_string())
                    .add_attribute(Attribute::Bold),
            ])
            .set_width(TERMINAL_MAX_WIDTH.into());

        // The top N commands and executables may have different lengths
        for i in 0..self.top_n_commands.len().max(self.top_n_executables.len()) {
            let rank_cell = Cell::new(format_rank_icon(i + 1)).set_alignment(CellAlignment::Center);

            let command_cell = self
                .top_n_commands
                .get(i)
                .map(|(cmd, count)| {
                    Cell::new(truncate_count_text(cmd, MAX_CELL_TEXT_LENGTH, *count))
                })
                .unwrap_or_else(|| Cell::new(""));

            let binary_cell = self
                .top_n_executables
                .get(i)
                .map(|(bin, count)| {
                    Cell::new(truncate_count_text(bin, MAX_CELL_TEXT_LENGTH, *count))
                })
                .unwrap_or_else(|| Cell::new(""));

            table.add_row(vec![rank_cell, command_cell, binary_cell]);
        }

        writeln!(f, "{table}")?;

        write!(f, "")
    }
}
