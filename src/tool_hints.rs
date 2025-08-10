//! Pure helpers for standardized tool-hint content shown to LLMs

/// Public helper to preview the standardized tool hint content.
/// This is a pure function (no async runtime needed) so tests can call it in #[test] contexts.
pub fn preview(operation_id: &str, operation_type: &str) -> String {
    format!(
        "\n\n### ASYNC CARGO OPERATION: {operation_type} (ID: {operation_id})\n\
1. **STATUS**: Operation is running in background - DO NOT assume it's complete\n\
2. **NEXT STEPS**:\n\
 - Continue your work (planning, coding, testing) while this runs\n\
 - When you need results, call: `mcp_async_cargo_m_wait` with operation_id='{operation_id}'\n\
 - For all pending operations: call `mcp_async_cargo_m_wait` without parameters\n\
\n+Next step: When you are ready to consume results, call `mcp_async_cargo_m_wait` with operation_id='{operation_id}'.\n\n

3. **IMPORTANT**:\n\
- Always use async_cargo_mcp MCP tools for ALL cargo operations\n\
 - Never run cargo directly in terminal\n\
 - Only wait for results when you're ready to use them\n\
 - You'll receive a notification with results when complete\n\n\
Wait only for the specific operation(s) needed for your next step.\n\n"
    )
}
