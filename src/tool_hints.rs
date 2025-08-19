//! Pure helpers for standardized tool-hint content shown to LLMs

/// Public helper to preview the standardized tool hint content.
/// This is a pure function (no async runtime needed) so tests can call it in #[test] contexts.
pub fn preview(operation_id: &str, operation_type: &str) -> String {
    // Updated to encourage explicit operation ID usage and batching before waiting
    format!(
        "\n\n### ASYNC CARGO OPERATION: {operation_type} (ID: {operation_id})\n\
1. **STATUS**: Operation is running in the background - DO NOT assume it's complete\n\
2. **NEXT STEPS**:\n\
 - Continue thinking about your next steps (planning, coding, testing, summarizing context if that is needed soon) while this tool runs\n\
 - Only when you have finished thinking and you need the results from this `{operation_type}`, call: `mcp_async_cargo_m_wait` with operation_ids=['{operation_id}']\n\
 - For multiple pending operations: call `mcp_async_cargo_m_wait` with operation_ids=['id1', 'id2', ...] to wait for them concurrently\n\
 - **Before you wait**: briefly decide what you'll do next and tell the user.\n\
 - **Consider outcomes**: if you expect success or failure, set expectations and outline follow-ups.\n\
 - **Batch actions**: send any other needed tool calls first, then wait for all relevant operation IDs at once.\n\
\n\
**Only call `wait` after you've queued your needed tools and collected all operation IDs you need to wait for.**\n\
\n\
+Next step: When you are ready to consume results, call `mcp_async_cargo_m_wait` with operation_ids=['{operation_id}'].\n\n\
3. **IMPORTANT**:\n\
 - Always use async_cargo_mcp MCP tools for ALL cargo operations\n\
 - Never run cargo directly in terminal\n\
 - Only wait for results when you're ready to use them\n\
 - Always specify explicit operation IDs - never leave operation_ids empty\n\
 - You'll receive a notification with results when complete\n\n\
Wait only for the specific operation(s) needed for your next step by providing their IDs explicitly.\n\n"
    )
}
