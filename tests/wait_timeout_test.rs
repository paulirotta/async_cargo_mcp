//! Test to verify that wait operations timeout correctly when waiting for operations that take too long
//! This test verifies the timeout behavior and default timeout value

use anyhow::Result;
mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::time::Instant;
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
async fn test_wait_timeout_for_long_running_operation() -> Result<()> {
    let temp = create_basic_project().await?;
    let working_dir = temp.path().to_str().unwrap().to_string();

    // Start server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Use deterministic sleep operations for timing sensitivity
    let sleep1 = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(object!({
                "duration_ms": 1500,
                "operation_id": "op_sleep_long_1"
            })),
        })
        .await?;
    let sleep2 = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(object!({
                "duration_ms": 1600,
                "operation_id": "op_sleep_long_2"
            })),
        })
        .await?;

    let operation_id1 = "op_sleep_long_1".to_string();
    let operation_id2 = "op_sleep_long_2".to_string();

    // Wait for both operations with a very short timeout (should timeout before tests complete)
    let start_time = Instant::now();
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": [operation_id1, operation_id2],
                "timeout_secs": 1  // 1 second timeout - should be too short for two test runs
            })),
        })
        .await?;
    let elapsed = start_time.elapsed();

    let wait_text = format!("{:?}", wait_result.content);

    // Should timeout after ~1 second
    assert!(
        elapsed.as_millis() >= 950 && elapsed.as_millis() < 1600,
        "Wait should have timed out near 1s, elapsed {:?}",
        elapsed
    );

    // Should contain timeout error message
    assert!(
        wait_text.contains("timed out") || wait_text.contains("timeout"),
        "Wait result should contain timeout message: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_wait_nonexistent_operation_returns_immediately() -> Result<()> {
    let temp = create_basic_project().await?;
    let _working_dir = temp.path().to_str().unwrap().to_string();

    // Start server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Try to wait for a non-existent operation - should return immediately with error
    let start_time = Instant::now();
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_never_exists_12345"],
                "timeout_secs": 30  // Long timeout, but should return immediately
            })),
        })
        .await?;
    let elapsed = start_time.elapsed();

    let wait_text = format!("{:?}", wait_result.content);

    // Should return immediately (within a few milliseconds)
    assert!(
        elapsed.as_millis() < 1000,
        "Wait for non-existent operation should return immediately, but took {:?}",
        elapsed
    );

    // Should contain "No operation found" error message
    assert!(
        wait_text.contains("No operation found") || wait_text.contains("OPERATION FAILED"),
        "Wait result should contain 'No operation found' error: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}
