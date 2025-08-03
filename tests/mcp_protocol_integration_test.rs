//! Comprehensive MCP Protocol Integration Test
//!
//! This test replicates the functionality of test-mcp.sh script but as a proper Rust integration test.
//! It tests the full MCP protocol flow including initialization, tool listing, and tool execution.
//!
//! Unlike the other integration tests that are ignored due to TokioChildProcess race conditions,
//! this test is designed to be more robust and provide comprehensive protocol validation.

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tokio::time::{Duration, sleep};

/// Comprehensive test that replicates test-mcp.sh functionality
#[tokio::test]
async fn test_mcp_protocol_comprehensive() -> Result<()> {
    // Create client connection to our MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test 1: Initialize (this happens automatically when we create the client)
    println!("✅ MCP initialization successful");

    // Small delay to ensure server is fully ready
    sleep(Duration::from_millis(100)).await;

    // Test 2: List available tools (equivalent to tools/list)
    let tools_result = client.list_all_tools().await?;
    println!(
        "✅ Tools list retrieved: {} tools available",
        tools_result.len()
    );

    // Verify expected tools are present
    let tool_names: Vec<String> = tools_result
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();

    assert!(
        tool_names.contains(&"say_hello".to_string()),
        "say_hello tool should be available"
    );
    assert!(
        tool_names.contains(&"echo".to_string()),
        "echo tool should be available"
    );
    assert!(
        tool_names.contains(&"sum".to_string()),
        "sum tool should be available"
    );
    assert!(
        tool_names.contains(&"build".to_string()),
        "build tool should be available"
    );
    assert!(
        tool_names.contains(&"check".to_string()),
        "check tool should be available"
    );

    // Verify counter tools are NOT present
    assert!(
        !tool_names.contains(&"increment".to_string()),
        "increment tool should not be available"
    );
    assert!(
        !tool_names.contains(&"decrement".to_string()),
        "decrement tool should not be available"
    );
    assert!(
        !tool_names.contains(&"get_value".to_string()),
        "get_value tool should not be available"
    );

    println!("✅ Tool availability validation passed");

    // Small delay between tool calls
    sleep(Duration::from_millis(50)).await;

    // Test 3: Test say_hello tool
    let hello_result = client
        .call_tool(CallToolRequestParam {
            name: "say_hello".into(),
            arguments: None,
        })
        .await?;

    assert!(
        !hello_result.is_error.unwrap_or(false),
        "say_hello should not return error"
    );
    assert!(
        !hello_result.content.is_empty(),
        "say_hello should return content"
    );
    println!("✅ say_hello tool test passed");

    // Small delay between tool calls
    sleep(Duration::from_millis(50)).await;

    // Test 4: Test echo tool with arguments
    let echo_result = client
        .call_tool(CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(object!({ "message": "Hello MCP!" })),
        })
        .await?;

    assert!(
        !echo_result.is_error.unwrap_or(false),
        "echo should not return error"
    );
    assert!(
        !echo_result.content.is_empty(),
        "echo should return content"
    );
    println!("✅ echo tool test passed");

    // Small delay between tool calls
    sleep(Duration::from_millis(50)).await;

    // Test 5: Test sum tool with arguments
    let sum_result = client
        .call_tool(CallToolRequestParam {
            name: "sum".into(),
            arguments: Some(object!({ "a": 5, "b": 3 })),
        })
        .await?;

    assert!(
        !sum_result.is_error.unwrap_or(false),
        "sum should not return error"
    );
    assert!(!sum_result.content.is_empty(), "sum should return content");
    println!("✅ sum tool test passed");

    // Small delay between tool calls
    sleep(Duration::from_millis(50)).await;

    // Test 6: Test cargo check command
    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: None,
        })
        .await?;

    // Note: check might return an error if run in the wrong directory, but it should still execute
    assert!(
        !check_result.content.is_empty(),
        "check should return some content"
    );
    println!("✅ check tool test completed");

    // Clean up
    let _ = client.cancel().await;

    println!("✅ All MCP protocol tests passed successfully!");
    Ok(())
}

/// Test that verifies the MCP protocol flow step by step
#[tokio::test]
async fn test_mcp_protocol_flow() -> Result<()> {
    // This test focuses on the protocol flow rather than tool functionality
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Get server info to verify connection
    let server_info = client.peer_info();
    if let Some(info) = server_info {
        println!("✅ Connected to server: {:?}", info);
    } else {
        println!("✅ Connected to server (info not available)");
    }

    // Test protocol capabilities
    let tools_result = client.list_all_tools().await?;
    assert!(tools_result.len() > 0, "Server should provide tools");
    println!("✅ Server provides {} tools", tools_result.len());

    // Test a simple tool call to verify the protocol works end-to-end
    let result = client
        .call_tool(CallToolRequestParam {
            name: "say_hello".into(),
            arguments: None,
        })
        .await?;

    assert!(!result.content.is_empty(), "Tool should return content");
    println!("✅ Protocol communication working");

    let _ = client.cancel().await;
    Ok(())
}
