use crate::utils::{TERMINAL_MAX_WIDTH, format_rank_icon, format_truncated};
use chrono::{Duration, NaiveDate};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use console::{measure_text_width, style};
use humanize_duration::Truncate;
use humanize_duration::prelude::DurationExt;
use std::fmt::{Display, Formatter};

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
