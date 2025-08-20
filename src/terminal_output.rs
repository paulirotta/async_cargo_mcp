//! Terminal output formatting and display utilities
//!
//! This module provides utilities for formatting cargo command output for terminal display,
//! including proper formatting, coloring, and JSON pretty-printing.

use serde_json::Value;
use std::io::{self, Write};

/// Terminal output utility for formatting and displaying cargo command results
pub struct TerminalOutput;

impl TerminalOutput {
    /// Write a formatted result to terminal with proper headers and formatting
    pub fn display_result(operation_id: &str, command: &str, description: &str, content: &str) {
        if content.trim().is_empty() {
            return; // Skip whitespace-only content
        }

        let mut stdout = io::stdout();
        let _ = stdout.write_all(b"\n");
        let _ = stdout.write_all(format!("=== {} ===\n", operation_id.to_uppercase()).as_bytes());
        let _ = stdout.write_all(format!("Command: {}\n", command).as_bytes());
        let _ = stdout.write_all(format!("Description: {}\n", description).as_bytes());
        let _ = stdout.write_all(b"\n");

        // Format the content
        let formatted_content = Self::format_content(content);
        let _ = stdout.write_all(formatted_content.as_bytes());
        let _ = stdout.write_all(b"\n");
        let _ = stdout.flush();
    }

    /// Format content with proper JSON pretty-printing and newline handling
    pub fn format_content(content: &str) -> String {
        // Try to parse as JSON first
        if let Ok(json_value) = serde_json::from_str::<Value>(content) {
            // Pretty print JSON with 2-space indentation
            if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                return pretty_json;
            }
        }

        // If not JSON, handle newlines and clean up formatting
        content
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .trim()
            .to_string()
    }

    /// Display multiple operation results from wait command
    pub fn display_wait_results(results: &[String]) {
        if results.is_empty() {
            return;
        }

        let mut stdout = io::stdout();
        let _ = stdout.write_all(b"\n");
        let _ =
            stdout.write_all(b"===============================================================\n");
        let _ =
            stdout.write_all(b"                    OPERATION RESULTS                         \n");
        let _ =
            stdout.write_all(b"===============================================================\n");
        let _ = stdout.write_all(b"\n");

        for (i, result) in results.iter().enumerate() {
            if i > 0 {
                let _ = stdout.write_all(
                    b"\n---------------------------------------------------------------\n\n",
                );
            }

            let formatted = Self::format_content(result);
            let _ = stdout.write_all(formatted.as_bytes());
            let _ = stdout.write_all(b"\n");
        }

        let _ =
            stdout.write_all(b"===============================================================\n");
        let _ = stdout.flush();
    }

    /// Check if content should be displayed (not just whitespace)
    pub fn should_display(content: &str) -> bool {
        !content.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_display() {
        assert!(!TerminalOutput::should_display(""));
        assert!(!TerminalOutput::should_display("   \n\t  "));
        assert!(TerminalOutput::should_display("some content"));
        assert!(TerminalOutput::should_display("  content  "));
    }

    #[test]
    fn test_format_content() {
        // Test JSON formatting
        let json_input = r#"{"name":"test","version":"1.0.0"}"#;
        let formatted = TerminalOutput::format_content(json_input);
        assert!(formatted.contains("{\n"));
        assert!(formatted.contains("  \"name\": \"test\""));

        // Test regular string formatting
        let string_input = "Hello\\nWorld\\tTab";
        let formatted = TerminalOutput::format_content(string_input);
        assert_eq!(formatted, "Hello\nWorld\tTab");
    }
}
