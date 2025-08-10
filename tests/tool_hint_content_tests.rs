//! Unit test for standardized tool-hint content

use async_cargo_mcp::tool_hints;

#[test]
fn tool_hint_preview_invariants() {
    let op_id = "op_123456789";
    let operation_type = "test";
    // Use the pure preview API which requires no async runtime
    let hint = tool_hints::preview(op_id, operation_type);

    // Invariants that should always hold, even if wording changes slightly
    let invariant_substrings = [
        "ASYNC CARGO OPERATION: ", // heading prefix
        operation_type,
        op_id, // includes operation id
        "STATUS",
        "NEXT STEPS",
        "IMPORTANT",
        "mcp_async_cargo_m_wait",
        "async_cargo_mcp", // tool family mention
        "Never run cargo directly in terminal",
    ];

    for needle in invariant_substrings {
        assert!(
            hint.contains(needle),
            "expected hint to contain: {needle}\nfull hint:\n{hint}"
        );
    }
}
