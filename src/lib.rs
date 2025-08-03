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

/// Test all the available tools and return a summary
///
/// This is a self-contained test function that creates its own client,
/// runs all tests, and cleans up.
pub async fn test_all_tools() -> Result<String> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test increment
    let inc_result = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Increment failed: {}", e))?;

    // Test get_value
    let value_result = client
        .call_tool(CallToolRequestParam {
            name: "get_value".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Get value failed: {}", e))?;

    // Test decrement
    let dec_result = client
        .call_tool(CallToolRequestParam {
            name: "decrement".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Decrement failed: {}", e))?;

    // Test echo
    let echo_result = client
        .call_tool(CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(object!({ "message": "test" })),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Echo failed: {}", e))?;

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
        "All tools tested successfully:\n- Increment: {:?}\n- Get Value: {:?}\n- Decrement: {:?}\n- Echo: {:?}\n- Sum: {:?}",
        inc_result, value_result, dec_result, echo_result, sum_result
    );

    // Cancel the client - ignore errors since transport might already be closed
    let _ = client.cancel().await;

    Ok(result)
}

/// Test increment functionality
pub async fn test_increment_functionality() -> Result<String> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Start with get_value to see initial state
    let initial = client
        .call_tool(CallToolRequestParam {
            name: "get_value".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Initial get_value failed: {}", e))?;

    // Increment twice
    let inc1 = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("First increment failed: {}", e))?;

    let inc2 = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Second increment failed: {}", e))?;

    // Get final value
    let final_value = client
        .call_tool(CallToolRequestParam {
            name: "get_value".into(),
            arguments: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Final get_value failed: {}", e))?;

    // Store the result before canceling the client
    let result = format!(
        "Increment test results:\n- Initial: {:?}\n- After first increment: {:?}\n- After second increment: {:?}\n- Final value: {:?}",
        initial, inc1, inc2, final_value
    );

    // Cancel the client - ignore errors since transport might already be closed
    let _ = client.cancel().await;

    Ok(result)
}
