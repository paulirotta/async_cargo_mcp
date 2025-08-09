//! Pure helpers for standardized tool-hint content shown to LLMs

/// Public helper to preview the standardized tool hint content.
/// This is a pure function (no async runtime needed) so tests can call it in #[test] contexts.
pub fn preview(operation_id: &str, operation_type: &str) -> String {
    format!(
        "\n\n*** CRITICAL Tool Hint for LLMs ***: Operation {operation_id} is running in the background.\n\
        *** DO NOT PROCEED assuming the operation is complete based on this message alone! ***\n\
        *** You must wait for completion to get actual results (success/failure/output)! ***\n\
        To get actual results, use:\n\
        • `mcp_async_cargo_m_wait` with operation_id='{operation_id}' to wait for this specific operation\n\
        • `mcp_async_cargo_m_wait` with no operation_id to wait for all pending operations\n\n\
        **Always use async_cargo_mcp MCP tools** instead of terminal commands for cargo operations.\n\
        You will receive progress notifications as the {operation_type} proceeds, but you MUST wait for completion."
    )
}
