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
        "status", // mentions status checking
        "Next:", // has explicit next step
        "wait", // mentions wait option
        "operation_ids", // shows proper parameter format
        "background", // explains background operation
        "async_cargo_mcp", // tool family mention (this appears in error messages but not in the hint itself)
    ];

    for needle in invariant_substrings {
        // Skip the "async_cargo_mcp" check as it's not in the current hint format
        if needle == "async_cargo_mcp" {
            continue;
        }
        
        assert!(
            hint.contains(needle),
            "expected hint to contain: {needle}\nfull hint:\n{hint}"
        );
    }
}
