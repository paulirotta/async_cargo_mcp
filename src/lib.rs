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
        .await?;

    // Test get_value
    let value_result = client
        .call_tool(CallToolRequestParam {
            name: "get_value".into(),
            arguments: None,
        })
        .await?;

    // Test decrement
    let dec_result = client
        .call_tool(CallToolRequestParam {
            name: "decrement".into(),
            arguments: None,
        })
        .await?;

    // Test echo
    let echo_result = client
        .call_tool(CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(object!({ "message": "test" })),
        })
        .await?;

    // Test sum
    let sum_result = client
        .call_tool(CallToolRequestParam {
            name: "sum".into(),
            arguments: Some(object!({ "a": 5, "b": 3 })),
        })
        .await?;

    client.cancel().await?;

    Ok(format!(
        "All tools tested successfully:\n- Increment: {:?}\n- Get Value: {:?}\n- Decrement: {:?}\n- Echo: {:?}\n- Sum: {:?}",
        inc_result, value_result, dec_result, echo_result, sum_result
    ))
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
        .await?;

    // Increment twice
    let inc1 = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await?;

    let inc2 = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await?;

    // Get final value
    let final_value = client
        .call_tool(CallToolRequestParam {
            name: "get_value".into(),
            arguments: None,
        })
        .await?;

    client.cancel().await?;

    Ok(format!(
        "Increment test results:\n- Initial: {:?}\n- After first increment: {:?}\n- After second increment: {:?}\n- Final value: {:?}",
        initial, inc1, inc2, final_value
    ))
}
