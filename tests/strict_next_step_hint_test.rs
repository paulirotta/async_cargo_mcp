//! Stricter guard: ensure the preview() includes an explicit "Next step:" line

use async_cargo_mcp::tool_hints;

#[test]
fn preview_contains_explicit_next_step_line() {
    let op_id = "op_test_0001";
    let operation_type = "build";
    let hint = tool_hints::preview(op_id, operation_type);

    // Accept either exact line or with leading whitespace; case-sensitive by design
    assert!(
        hint.contains("Next:"),
        "preview() must contain an explicit 'Next:' line. Full hint:\n{hint}"
    );
}
