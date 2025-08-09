//! Unit test for standardized tool-hint content

use async_cargo_mcp::tool_hints;

#[test]
fn tool_hint_contains_required_phrases() {
    let op_id = "op_123456789";
    // Use the pure preview API which requires no async runtime
    let hint = tool_hints::preview(op_id, "test");

    let required = [
        "CRITICAL Tool Hint for LLMs",
        "running in the background",
        "DO NOT PROCEED",
        "You must wait for completion",
        "mcp_async_cargo_m_wait",
        op_id,
        "async_cargo_mcp MCP tools",
    ];

    for needle in required {
        assert!(
            hint.contains(needle),
            "expected hint to contain: {needle}\nfull hint:\n{hint}"
        );
    }
}
