//! Verify async build returns full build output via wait and progress notifications
//!
//! This test ensures that when enable_async_notifications=true is used, the initial
//! build response is a tool hint with an operation_id, and calling `wait` returns
//! the full captured output from cargo build.

use anyhow::Result;
mod common;
use async_cargo_mcp::tool_hints;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

#[tokio::test]
async fn test_async_build_then_wait_returns_full_output() -> Result<()> {
    // Create a minimal cargo project in a temp dir
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server via cargo run so the binary used is current
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Kick off async build with enable_async_notifications=true
    let build_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": true
            })),
        })
        .await?;

    let first_text = format!("{:?}", build_result.content);
    // Should include a hint and an operation id string we can extract
    assert!(first_text.contains("started in background"));

    // Extract operation id using the known prefix `op_`
    let op_id = extract_operation_id(&first_text).expect("operation id should be present");
    assert!(op_id.starts_with("op_"));

    // The initial async response should include the standardized preview() hint.
    // first_text is built via Debug formatting (escapes newlines), so accept either raw or escaped forms.
    let expected_hint = tool_hints::preview(&op_id, "build");
    let expected_hint_debug = expected_hint.replace('\n', "\\n");
    assert!(
        first_text.contains(&expected_hint_debug) || first_text.contains(&expected_hint),
        "Initial async response must include preview() content.\nExpected preview (raw or debug-escaped):\n{expected_hint}\n--- Escaped ---\n{expected_hint_debug}\nGot:\n{first_text}"
    );

    // Now wait for the operation to complete and verify the output content
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);
    // Expect our formatted result from wait() with the FULL OUTPUT marker
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));
    assert!(
        wait_text.contains("=== FULL"),
        "Wait output should contain full output marker: {wait_text}"
    );
    // And at least some typical cargo build line
    assert!(
        wait_text.contains("Compiling")
            || wait_text.contains("Finished")
            || wait_text.contains("Building"),
        "Expected cargo build lines in wait output: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

fn extract_operation_id(s: &str) -> Option<String> {
    // crude parse: find "op_" and take until whitespace or punctuation
    if let Some(start) = s.find("op_") {
        let rest = &s[start..];
        let mut id = String::new();
        for ch in rest.chars() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                id.push(ch);
            } else {
                break;
            }
        }
        if id.starts_with("op_") {
            return Some(id);
        }
    }
    None
}

// moved to tests/common/test_project.rs
