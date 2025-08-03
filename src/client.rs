//! MCP client for testing and demonstrating the async_cargo_mcp server
//!
//! This module provides a simple client application that connects to the async_cargo_mcp
//! server and demonstrates how to interact with the available MCP tools. It serves as both
//! a testing utility and an example of how external clients can integrate with the server.
//!
//! ## Features
//!
//! - Establishes connection to the async_cargo_mcp server as a child process
//! - Lists all available tools and their capabilities
//! - Demonstrates tool invocation patterns for testing and validation
//! - Provides logging and error handling examples for client implementations
//!
//! ## Usage
//!
//! Run this client directly to test server functionality:
//!
//! ```bash
//! cargo run --bin client
//! ```
//!
//! This client will start the server, connect to it, list available tools,
//! and execute sample commands to verify everything is working correctly.

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    //TODO is this coming back to the vscode server on stderr causing problems when it expects only json?
    tracing::info!("Starting MCP client to test our server");

    // Start our server as a child process
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    tracing::info!("Connected to server successfully!");

    // Initialize and get server info
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");

    // List tools
    let tools = client.list_all_tools().await?;
    tracing::info!("Available tools: {tools:#?}");

    // Call increment tool
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await?;
    tracing::info!("Tool result for increment: {tool_result:#?}");

    // Call increment again
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: None,
        })
        .await?;
    tracing::info!("Tool result for second increment: {tool_result:#?}");

    // Get current value
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "get_value".into(),
            arguments: None,
        })
        .await?;
    tracing::info!("Tool result for get_value: {tool_result:#?}");

    // Test decrement
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "decrement".into(),
            arguments: None,
        })
        .await?;
    tracing::info!("Tool result for decrement: {tool_result:#?}");

    // Test echo with parameters
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(object!({ "message": "Hello from client!" })),
        })
        .await?;
    tracing::info!("Tool result for echo: {tool_result:#?}");

    // Test sum with parameters
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "sum".into(),
            arguments: Some(object!({ "a": 5, "b": 3 })),
        })
        .await?;
    tracing::info!("Tool result for sum: {tool_result:#?}");

    // Test cargo build with async notifications
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({ "enable_async_notifications": true })),
        })
        .await?;
    tracing::info!("Tool result for cargo build with async notifications: {tool_result:#?}");

    // Test cargo add with async notifications
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "add".into(),
            arguments: Some(object!({
                "name": "serde",
                "version": "1.0",
                "enable_async_notifications": true
            })),
        })
        .await?;
    tracing::info!("Tool result for cargo add with async notifications: {tool_result:#?}");

    client.cancel().await?;

    tracing::info!("Client test completed successfully!");
    Ok(())
}
