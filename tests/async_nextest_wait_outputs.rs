//! Verify async nextest returns full output via wait and progress notifications
//!
//! This mirrors the build test but exercises the cargo-nextest path.

use anyhow::Result;
use async_cargo_mcp::tool_hints;
mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
// imports adjusted after moving project creation to common helpers

#[tokio::test]
async fn test_async_nextest_then_wait_returns_full_output() -> Result<()> {
    // Create a minimal cargo project with a single test
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

    // Kick off async nextest with enable_async_notifications=true
    let nextest_result = client
        .call_tool(CallToolRequestParam {
            name: "nextest".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": true,
                // keep runs short/reliable
                "args": ["--no-fail-fast"]
            })),
        })
        .await?;

    let first_text = format!("{:?}", nextest_result.content);
    // Should include a hint and an operation id string we can extract
    assert!(first_text.contains("started in background"));

    // Extract operation id using the known prefix `op_`
    let op_id = extract_operation_id(&first_text).expect("operation id should be present");
    assert!(op_id.starts_with("op_"));

    // Verify the standardized preview() hint is included (accept raw or debug-escaped forms)
    let expected_hint = tool_hints::preview(&op_id, "nextest");
    let expected_hint_debug = expected_hint.replace('\n', "\\n");
    assert!(
        first_text.contains(&expected_hint_debug) || first_text.contains(&expected_hint),
        "Initial async response must include preview() content.\nExpected preview (raw or debug-escaped):\n{expected_hint}\n--- Escaped ---\n{expected_hint_debug}\nGot:\n{first_text}"
    );

    // Now wait for the operation to complete and verify the output content
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_id": op_id, "timeout_secs": 180 })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);
    // Expect our formatted result from wait() with the FULL OUTPUT marker
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));
    assert!(
        wait_text.contains("=== FULL"),
        "Wait output should contain full output marker: {wait_text}"
    );

    // New stricter assertions: nextest should show real test output, not just placeholder
    assert!(
        !wait_text.contains("(no command stdout captured") || wait_text.contains("test result"),
        "Nextest wait output appears empty / placeholder instead of real test output: {wait_text}"
    );
    assert!(
        wait_text.contains("test") || wait_text.contains("running"),
        "Expected some nextest test run indicators in output: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

fn extract_operation_id(s: &str) -> Option<String> {
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
