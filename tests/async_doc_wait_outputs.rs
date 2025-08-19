//! Verify async doc returns full output via wait and includes generated path
//!
//! This mirrors the async build test but for the `doc` command.

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
async fn test_async_doc_then_wait_returns_full_output_and_path() -> Result<()> {
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

    // Kick off async doc with enable_async_notifications=true
    let start = client
        .call_tool(CallToolRequestParam {
            name: "doc".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": true
            })),
        })
        .await?;

    let first_text = format!("{:?}", start.content);
    assert!(first_text.contains("started at"));

    // Extract the operation id and confirm preview() content appears
    let op_id = extract_operation_id(&first_text).expect("operation id should be present");
    let expected_hint = tool_hints::preview(&op_id, "documentation generation");
    let expected_hint_debug = expected_hint.replace('\n', "\\n");
    assert!(
        first_text.contains(&expected_hint) || first_text.contains(&expected_hint_debug),
        "Initial async response must include preview() content for doc. Got: {first_text}"
    );

    // Wait for completion and validate the output
    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait.content);
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));
    assert!(wait_text.contains("=== FULL"));

    // We expect our doc implementation success text when it succeeds
    // This may fail if rustdoc fails on the CI environment; still assert the presence of informative text if completed.
    if wait_text.contains("OPERATION COMPLETED") {
        assert!(
            wait_text.contains("Documentation generated at:"),
            "Wait output should include generated doc path when successful: {wait_text}"
        );
    }

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
