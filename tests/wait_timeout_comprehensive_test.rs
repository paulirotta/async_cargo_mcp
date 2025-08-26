//! Test comprehensive timeout behavior for wait operations
//! This test covers the case where wait operations should timeout when operations don't exist or take too long

use anyhow::Result;
mod common;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::time::Instant;
use tokio::process::Command;

#[tokio::test]
async fn test_wait_timeout_for_nonexistent_operation() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    let start = Instant::now();

    // Try to wait for a nonexistent operation with 2 second timeout
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_nonexistent_abc123"]
            })),
        })
        .await?;

    let elapsed = start.elapsed();
    let wait_text = format!("{:?}", wait_result.content);

    // Should return immediately with error info about nonexistent operation
    assert!(
        elapsed.as_secs() < 2,
        "Wait should return immediately for nonexistent operation, not wait for timeout. Elapsed: {:?}",
        elapsed
    );

    // Should contain information about the missing operation
    assert!(
        wait_text.contains("No operation found") || wait_text.contains("nonexistent"),
        "Wait result should mention the nonexistent operation: {}",
        wait_text
    );

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_wait_timeout_for_empty_operation_list() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    let start = Instant::now();

    // Try to wait for empty operation list - should fail immediately
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": []
            })),
        })
        .await;

    let elapsed = start.elapsed();

    // Should fail immediately with validation error
    assert!(
        elapsed.as_secs() < 1,
        "Wait should fail immediately for empty operation_ids. Elapsed: {:?}",
        elapsed
    );

    // Should be an error result
    assert!(
        wait_result.is_err(),
        "Wait with empty operation_ids should return an error, got: {:?}",
        wait_result
    );

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_wait_default_timeout_is_reasonable() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    let start = Instant::now();

    // Try to wait for a nonexistent operation WITHOUT specifying timeout
    // This should use the default timeout
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_nonexistent_default_timeout_test"]
            })),
        })
        .await;

    assert!(
        wait_result.is_ok(),
        "Wait should succeed even for nonexistent operation with default timeout"
    );

    let elapsed = start.elapsed();

    // Should return immediately (not wait for default timeout) for nonexistent operations
    assert!(
        elapsed.as_secs() < 5,
        "Wait should return immediately for nonexistent operations, not use default timeout. Elapsed: {:?}",
        elapsed
    );

    let _ = client.cancel().await;
    Ok(())
}
