//! Library module for async_cargo_mcp
//!
//! This module exposes the client functionality for integration tests

pub mod callback_system;
pub mod cargo_tools;
pub mod command_registry;
pub mod operation_monitor;
pub mod test_cargo_tools;

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tokio::time::{Duration, sleep};

/// Test the remaining utility tools (echo, sum, say_hello)
///
/// This is a self-contained test function that creates its own client,
/// runs all non-counter tests, and cleans up.
pub async fn test_utility_tools() -> Result<String> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test say_hello
    let hello_result = client
        .call_tool(CallToolRequestParam {
            name: "say_hello".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Say hello failed: {}", e))?;

    // Small delay to prevent TokioChildProcess race condition
    sleep(Duration::from_millis(50)).await;

    // Test echo
    let echo_result = client
        .call_tool(CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(object!({ "message": "test" })),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Echo failed: {}", e))?;

    // Small delay to prevent TokioChildProcess race condition
    sleep(Duration::from_millis(50)).await;

    // Test sum
    let sum_result = client
        .call_tool(CallToolRequestParam {
            name: "sum".into(),
            arguments: Some(object!({ "a": 5, "b": 3 })),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Sum failed: {}", e))?;

    // Store the result before canceling the client
    let result = format!(
        "Utility tools tested successfully:\n- Say Hello: {:?}\n- Echo: {:?}\n- Sum: {:?}",
        hello_result, echo_result, sum_result
    );

    // Cancel the client - ignore errors since transport might already be closed
    let _ = client.cancel().await;

    Ok(result)
}
