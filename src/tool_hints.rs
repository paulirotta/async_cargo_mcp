//! Pure helpers for standardized tool-hint content shown to LLMs

/// Public helper to preview the standardized tool hint content.
/// This is a pure function (no async runtime needed) so tests can call it in #[test] contexts.
pub fn preview(operation_id: &str, operation_type: &str) -> String {
    use crate::constants::TOOL_HINT_TEMPLATE;
    TOOL_HINT_TEMPLATE
        .replace("{operation_type}", operation_type)
        .replace("{operation_id}", operation_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_replaces_placeholders() {
        let out = preview("abc123", "build");
        assert!(out.contains("abc123"));
        assert!(out.contains("build"));
        assert!(!out.contains("{operation_id}"));
        assert!(!out.contains("{operation_type}"));
    }
}
