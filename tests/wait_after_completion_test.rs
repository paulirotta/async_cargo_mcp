//! Test that verifies waiting for already completed operations returns cached results
//!
//! This addresses the issue where LLMs might call wait for operations that have already
//! finished, and we need to return the stored results instead of timing out or failing.

use anyhow::Result;
mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

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

#[tokio::test]
async fn test_wait_after_completion_returns_cached_results() -> Result<()> {
    // Create a minimal cargo project in a temp dir
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Start an async operation
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
    let op_id = extract_operation_id(&first_text).expect("operation id should be present");

    // Wait for the operation to complete (first wait call)
    let first_wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id.clone()], "timeout_secs": 120 })),
        })
        .await?;

    let first_wait_text = format!("{:?}", first_wait_result.content);
    assert!(
        first_wait_text.contains("OPERATION COMPLETED")
            || first_wait_text.contains("OPERATION FAILED")
    );

    // Give a small delay to ensure the operation is fully processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Now wait for the SAME operation again (this should return cached results immediately)
    let second_wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id.clone()], "timeout_secs": 5 })),
        })
        .await?;

    let second_wait_text = format!("{:?}", second_wait_result.content);

    // Should still get the completed operation result
    assert!(
        second_wait_text.contains("OPERATION COMPLETED")
            || second_wait_text.contains("OPERATION FAILED")
    );

    // The result should be essentially the same (cached result)
    // Both should contain the full output marker
    assert!(first_wait_text.contains("=== FULL"));
    assert!(second_wait_text.contains("=== FULL"));

    // Both should reference the same operation ID
    assert!(first_wait_text.contains(&op_id));
    assert!(second_wait_text.contains(&op_id));

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_wait_for_nonexistent_operation_provides_helpful_message() -> Result<()> {
    // Create a minimal cargo project in a temp dir
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Try to wait for a non-existent operation ID
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(
                object!({ "operation_ids": ["op_nonexistent_12345"], "timeout_secs": 5 }),
            ),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Should get a helpful message about the missing operation
    assert!(wait_text.contains("No operation found"));
    assert!(wait_text.contains("op_nonexistent_12345"));

    // Should provide guidance about what might have happened
    assert!(wait_text.contains("cleaned up") || wait_text.contains("incorrect"));

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_wait_for_multiple_operations_including_nonexistent() -> Result<()> {
    // Create a minimal cargo project in a temp dir
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Start a real async operation
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
    let real_op_id = extract_operation_id(&first_text).expect("operation id should be present");

    // Wait for both a real operation and a fake one
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": [real_op_id.clone(), "op_fake_999"],
                "timeout_secs": 120
            })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Should get results for both operations
    // One should be completed/failed (the real one)
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));

    // The other should be a helpful message about the missing operation
    assert!(wait_text.contains("No operation found") && wait_text.contains("op_fake_999"));

    // Should contain both operation IDs
    assert!(wait_text.contains(&real_op_id));
    assert!(wait_text.contains("op_fake_999"));

    let _ = client.cancel().await;
    Ok(())
}
