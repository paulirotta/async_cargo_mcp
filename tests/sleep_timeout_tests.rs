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
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
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
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
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

    // Wait with 1s timeout
    let start = Instant::now();
    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_sleep_long"],
                "timeout_secs": 1
            })),
        })
        .await?;
    let elapsed = start.elapsed();
    let wait_text = format!("{:?}", wait.content);

    assert!(
        elapsed.as_secs() >= 1 && elapsed.as_secs() < 2,
        "elapsed {:?}",
        elapsed
    );
    assert!(wait_text.contains("timed out") || wait_text.contains("TIMEOUT"));

    // Now wait again with larger timeout and ensure completion
    let wait2 = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": ["op_sleep_long"],
                "timeout_secs": 5
            })),
        })
        .await?;
    let wait2_text = format!("{:?}", wait2.content);
    assert!(wait2_text.contains("Slept for 2000ms"));

    let _ = client.cancel().await;
    Ok(())
}
