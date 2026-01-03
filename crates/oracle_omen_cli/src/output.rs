//! Output formatting for CLI.

use std::fmt;

/// Formatted output
pub struct Output {
    /// Output lines
    lines: Vec<String>,
}

impl Output {
    /// Create new output
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a line
    pub fn line(mut self, line: impl fmt::Display) -> Self {
        self.lines.push(line.to_string());
        self
    }

    /// Add a header
    pub fn header(mut self, text: impl fmt::Display) -> Self {
        self.lines.push(format!("=== {} ===", text));
        self
    }

    /// Add a section
    pub fn section(mut self, text: impl fmt::Display) -> Self {
        self.lines.push(format!("\n-- {} --", text));
        self
    }

    /// Add key-value pair
    pub fn kv(mut self, key: impl fmt::Display, value: impl fmt::Display) -> Self {
        self.lines.push(format!("{}: {}", key, value));
        self
    }

    /// Build the output string
    pub fn build(self) -> String {
        self.lines.join("\n")
    }

    /// Print to stdout
    pub fn print(self) {
        println!("{}", self.build());
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

/// Table formatter
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    /// Create a new table
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    /// Add a row
    pub fn row(mut self, values: Vec<String>) -> Self {
        self.rows.push(values);
        self
    }

    /// Format the table
    pub fn format(self) -> String {
        if self.rows.is_empty() {
            return self.headers.join(" | ");
        }

        // Calculate column widths
        let mut widths = vec
![0usize; self.headers.len()
];
        for (i, h) in self.headers.iter().enumerate() {
            widths[i] = widths[i].max(h.len());
        }
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        let mut result = Vec::new();

        // Header
        let header_row: Vec<String> = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
            .collect();
        result.push(header_row.join(" | "));

        // Separator
        let sep: String = widths
            .iter()
            .map(|w| "-".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("+");
        result.push(format!("+{}+", sep));

        // Rows
        for row in &self.rows {
            let formatted: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    if i < widths.len() {
                        format!("{:<width$}", cell, width = widths[i])
                    } else {
                        cell.clone()
                    }
                })
                .collect();
            result.push(formatted.join(" | "));
        }

        result.join("\n")
    }

    /// Print the table
    pub fn print(self) {
        println!("{}", self.format());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output() {
        let out = Output::new()
            .header("Test")
            .line("Line 1")
            .line("Line 2")
            .build();
        assert!(out.contains("=== Test ==="));
        assert!(out.contains("Line 1"));
    }

    #[test]
    fn test_table() {
        let table = Table::new(vec!["A".to_string(), "B".to_string()])
            .row(vec!["1".to_string(), "2".to_string()])
            .row(vec!["3".to_string(), "4".to_string()])
            .format();
        assert!(table.contains("A | B"));
        assert!(table.contains("1 | 2"));
    }
}
