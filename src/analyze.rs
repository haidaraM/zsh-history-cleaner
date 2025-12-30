use crate::history::History;
use crate::utils::{TERMINAL_MAX_WIDTH, format_rank_icon, format_truncated};
use chrono::{Duration, Local, NaiveDate};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use console::{measure_text_width, style};
use humanize_duration::Truncate;
use humanize_duration::prelude::DurationExt;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

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

    /// Return the top n most frequent commands.
    /// If n is 0, returns an empty vector.
    pub fn top_n_commands(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 || self.history.entries().is_empty() {
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

    /// Return the top n most frequent binaries (first word of the command).
    /// If n is 0, returns an empty vector.
    pub fn top_n_binaries(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 || self.history.entries().is_empty() {
            return Vec::new();
        }

        // Count occurrences of each binary (first word of the command)
        let mut binaries_count: HashMap<&str, usize> = HashMap::new();

        for entry in self.history.entries() {
            if let Some(command) = entry.valid_command()
                && let Some(binary) = command.split_whitespace().next()
            {
                *binaries_count.entry(binary).or_insert(0) += 1;
            }
        }

        let mut count_vec: Vec<(&str, usize)> = binaries_count.into_iter().collect();
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

    /// Analyze the History and return a HistoryAnalysis struct
    pub fn analyze(&self, n: usize) -> HistoryAnalysis {
        let date_range = self.date_range().unwrap_or_else(|| {
            let now = Local::now().date_naive();
            (now, now)
        });
        HistoryAnalysis {
            filename: self.history.filename().to_string(),
            size: self.history.size(),
            date_range,
            top_n_commands: self.top_n_commands(n),
            top_n_binaries: self.top_n_binaries(n),
        }
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
    /// The top N most frequent commands
    pub top_n_commands: Vec<(String, usize)>,
    /// The top N most frequent binaries
    pub top_n_binaries: Vec<(String, usize)>,
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
        let title = format!(
            "📊 History Analysis for {}",
            style(&self.filename).cyan().bold()
        );

        // Format date range with colored dates
        let date_range_text = format!(
            "🗓️ {} → {} {}",
            style(&self.date_range.0).green().bold(),
            style(&self.date_range.1).green().bold(),
            style(format!("({})", human_duration)).dim()
        );

        // Format total commands with highlighted number
        let total_commands = format!("📝 Total Commands: {}", style(&self.size).yellow().bold());

        // Print the stats box with properly aligned borders
        writeln!(f, "{}", style(top_border).blue())?;
        writeln!(f, "{}", make_box_line(title))?;
        writeln!(f, "{}", make_box_line(date_range_text))?;
        writeln!(f, "{}", make_box_line(total_commands))?;
        writeln!(f, "{}", style(bottom_border).blue())?;
        writeln!(f)?;

        // Section header for top items
        writeln!(
            f,
            "{} {}",
            style("🔥").bold(),
            style(format!(
                "Top {} Most Used:",
                self.top_n_commands.len().max(self.top_n_binaries.len())
            ))
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
                Cell::new(style("Binaries").cyan().bold().to_string())
                    .add_attribute(Attribute::Bold),
            ])
            .set_width(TERMINAL_MAX_WIDTH.into());

        // The top N commands and binaries may have different lengths
        for i in 0..self.top_n_commands.len().max(self.top_n_binaries.len()) {
            let rank_cell = Cell::new(format_rank_icon(i + 1));

            let command_cell = self
                .top_n_commands
                .get(i)
                .map(|(cmd, count)| Cell::new(format_truncated(cmd, 39, *count)))
                .unwrap_or_else(|| Cell::new(""));

            let binary_cell = self
                .top_n_binaries
                .get(i)
                .map(|(bin, count)| Cell::new(format_truncated(bin, 39, *count)))
                .unwrap_or_else(|| Cell::new(""));

            table.add_row(vec![rank_cell, command_cell, binary_cell]);
        }

        writeln!(f, "{table}")?;

        write!(f, "")
    }
}
