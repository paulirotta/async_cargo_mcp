//! Tests for deterministic sleep tool to validate timeout and success scenarios

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
async fn test_sleep_operation_completes() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    let sleep_result = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(object!({
                "duration_ms": 300,
                "operation_id": "op_sleep_short"
            })),
        })
        .await?;
    let text = format!("{:?}", sleep_result.content);
    assert!(text.contains("op_sleep_short"));

    // Wait long enough for it to finish
    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_sleep_short"]
            })),
        })
        .await?;
    let wait_text = format!("{:?}", wait.content);
    assert!(wait_text.contains("Slept for 300ms"));

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_sleep_operation_timeout() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Start 2s sleep
    let sleep_result = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(object!({
                "duration_ms": 2000,
                "operation_id": "op_sleep_long"
            })),
        })
        .await?;
    let text = format!("{:?}", sleep_result.content);
    assert!(text.contains("op_sleep_long"));

    // Wait with fixed 300s timeout - for a 2s sleep this should succeed
    let start = Instant::now();
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_sleep_long"]
            })),
        })
        .await;
    let elapsed = start.elapsed();

    // The wait should now succeed since we're using fixed 300s timeout for a 2s sleep
    assert!(
        wait_result.is_ok(),
        "Expected wait to succeed with fixed 300s timeout for 2s sleep, got error: {:?}",
        wait_result.unwrap_err()
    );
    let wait_text = format!("{:?}", wait_result.unwrap().content);
    assert!(wait_text.contains("Slept for 2000ms"));

    assert!(
        elapsed.as_secs() >= 2 && elapsed.as_secs() < 4,
        "elapsed should be around 2s for the sleep, got: {:?}",
        elapsed
    );

    let _ = client.cancel().await;
    Ok(())
}
