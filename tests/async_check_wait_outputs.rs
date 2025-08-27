//! Verify async check returns full output via wait and progress notifications

use anyhow::Result;
use async_cargo_mcp::{test_utils, tool_hints};
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
async fn test_async_check_then_wait_returns_full_output() -> Result<()> {
    // Create a minimal cargo project
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server via cargo run
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
            },
        ))?)
        .await?;

    // Kick off async check
    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let first_text = format!("{:?}", check_result.content);
    // Should include a hint and an operation id string we can extract
    assert!(first_text.contains("started at"));

    let op_id = extract_operation_id(&first_text).expect("operation id should be present");
    assert!(op_id.starts_with("op_"));

    // Verify the standardized preview() hint is included (accept raw or debug-escaped forms)
    let expected_hint = tool_hints::preview(&op_id, "check");
    assert!(
        test_utils::includes(&first_text, &expected_hint),
        "Initial async response must include preview() content.\nExpected preview:\n{expected_hint}\nGot:\n{first_text}"
    );

    // Wait for completion
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));
    assert!(
        wait_text.contains("=== FULL"),
        "Wait output should contain full output marker: {wait_text}"
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
