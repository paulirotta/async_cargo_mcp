//! Comprehensive MCP Protocol Integration Test
//!
//! This test replicates the functionality of test-mcp.sh script but as a proper Rust integration test.
//! It tests the full MCP protocol flow including initialization, tool listing, and tool execution.
//!
//! Unlike the other integration tests that are ignored due to TokioChildProcess race conditions,
//! this test is designed to be more robust and provide comprehensive protocol validation.

use anyhow::Result;
mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::json;
// imports adjusted after moving project creation to common helpers
use tokio::process::Command;
use tokio::time::{Duration, sleep};

/// Comprehensive test that replicates test-mcp.sh functionality
#[tokio::test]
async fn test_mcp_protocol_comprehensive() -> Result<()> {
    // Create a temporary project for testing add/remove operations
    let temp_project = create_basic_project().await?;
    let temp_project_path = temp_project.path().to_str().unwrap();

    // Create client connection to our MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test 1: Initialize (happens automatically) and ensure GPT-5 notice is present in server info
    let server_info = client.peer_info();
    if let Some(info) = &server_info {
        if let Some(instr) = &info.instructions {
            assert!(
                instr.contains("GPT-5 (Preview)"),
                "Initialize instructions should mention GPT-5 (Preview). Got: {}",
                instr
            );
        } else {
            panic!("Server info did not include instructions field");
        }
    } else {
        panic!("peer_info() returned None; expected Some(ServerInfo)");
    }
    println!("MCP initialization successful (GPT-5 Preview notice detected)");

    // Small delay to ensure server is fully ready
    sleep(Duration::from_millis(100)).await;

    // Test 2: List available tools (equivalent to tools/list)
    let tools_result = client.list_all_tools().await?;
    println!(
        "Tools list retrieved: {} tools available",
        tools_result.len()
    );

    // Define the complete set of expected cargo tools
    // Note: We exclude "test" to avoid recursive execution during testing
    let expected_tools = vec!["add", "build", "check", "doc", "remove", "run", "update"];

    // Verify ALL expected tools are present
    let tool_names: Vec<String> = tools_result
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();

    println!("Available tools: {tool_names:?}");

    for expected_tool in &expected_tools {
        assert!(
            tool_names.contains(&expected_tool.to_string()),
            "{expected_tool} tool should be available but was not found in: {tool_names:?}"
        );
    }

    // Verify we don't have unexpected tools (like the old utility functions)
    let unexpected_tools = vec![
        "say_hello",
        "echo",
        "sum",
        "increment",
        "decrement",
        "get_value",
    ];
    for unexpected_tool in &unexpected_tools {
        assert!(
            !tool_names.contains(&unexpected_tool.to_string()),
            "Unexpected utility tool '{unexpected_tool}' found - these should have been removed"
        );
    }

    // Verify we have exactly the expected number of tools (catches if new tools are added)
    // We expect 28 total tools including new commands: fmt, tree, version, fetch, rustc, metadata, wait, sleep, cargo_lock_remediation, and bump_version
    assert_eq!(
        tool_names.len(),
        28,
        "Expected exactly 28 tools, but found {}. Tools: {:?}",
        tool_names.len(),
        tool_names
    );

    // Verify the test tool exists but we won't execute it to avoid recursion
    assert!(
        tool_names.contains(&"test".to_string()),
        "test tool should be available but was not found in: {tool_names:?}"
    );

    println!(
        "Tool availability validation passed - all {} expected cargo tools present",
        expected_tools.len()
    );

    // Test 3: Test each cargo tool to ensure they execute without protocol errors
    // Note: We expect some tools to return errors when run in the wrong context,
    // but they should still execute and return proper MCP responses

    for tool_name in &expected_tools {
        sleep(Duration::from_millis(50)).await; // Small delay between calls

        // Provide appropriate arguments for tools that require them
        let arguments = match &**tool_name {
            "add" => {
                let obj = json!({
                    "name": "serde",
                    "working_directory": temp_project_path
                });
                obj.as_object().cloned()
            }
            "remove" => {
                let obj = json!({
                    "name": "serde",
                    "working_directory": temp_project_path
                });
                obj.as_object().cloned()
            }
            _ => {
                // For all other tools, also use the temp project directory
                // to avoid affecting the main project
                let obj = json!({
                    "working_directory": temp_project_path
                });
                obj.as_object().cloned()
            }
        };

        let result = client
            .call_tool(CallToolRequestParam {
                name: (*tool_name).into(),
                arguments,
            })
            .await?;

        // Verify the tool returned some content (even if it's an error message)
        assert!(
            !result.content.is_empty(),
            "{tool_name} tool should return some content"
        );

        // Verify the result has the expected structure
        assert!(
            !result.content.is_empty(),
            "{tool_name} tool should return at least one content item"
        );

        println!("{tool_name} tool executed successfully");
    }

    // Test 4: Verify tool descriptions are present and meaningful
    for tool in &tools_result {
        assert!(
            tool.description.is_some(),
            "Tool '{}' should have a description",
            tool.name
        );

        let desc = tool.description.as_ref().unwrap();
        assert!(
            !desc.is_empty(),
            "Tool '{}' description should not be empty",
            tool.name
        );

        // Verify description mentions cargo (since all our tools are cargo-related)
        assert!(
            desc.to_lowercase().contains("cargo"),
            "Tool '{}' description should mention cargo: '{}'",
            tool.name,
            desc
        );
    }

    println!("Tool descriptions validation passed");

    // Test 5: Test specific doc command functionality (since it's particularly important)
    let doc_result = client
        .call_tool(CallToolRequestParam {
            name: "doc".into(),
            arguments: {
                let obj = json!({
                    "working_directory": temp_project_path
                });
                obj.as_object().cloned()
            },
        })
        .await?;

    // Verify doc command returns meaningful content
    assert!(
        !doc_result.content.is_empty(),
        "doc command should return content"
    );

    // For doc command, verify it mentions documentation in the output
    let doc_output = format!("{:?}", doc_result.content);
    assert!(
        doc_output.to_lowercase().contains("documentation")
            || doc_output.to_lowercase().contains("doc"),
        "doc command output should mention documentation: {doc_output}"
    );

    println!("doc command specific validation passed");

    // Clean up
    let _ = client.cancel().await;

    println!("All MCP protocol tests passed successfully!");
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
    if let Some(info) = &server_info {
        if let Some(instr) = &info.instructions {
            assert!(
                instr.contains("GPT-5 (Preview)"),
                "Initialize instructions should mention GPT-5 (Preview). Got: {}",
                instr
            );
        }
        println!("Connected to server: {info:?}");
    }

    // Test protocol capabilities
    let tools_result = client.list_all_tools().await?;
    assert!(!tools_result.is_empty(), "Server should provide tools");
    println!("Server provides {} tools", tools_result.len());

    // Test a simple tool call to verify the protocol works end-to-end
    // Create a temporary project for this test
    let temp_project = create_basic_project().await?;
    let temp_project_path = temp_project.path().to_str().unwrap();

    let result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: {
                let obj = json!({
                    "working_directory": temp_project_path
                });
                obj.as_object().cloned()
            },
        })
        .await?;

    assert!(!result.content.is_empty(), "Tool should return content");
    println!("Protocol communication working");

    let _ = client.cancel().await;
    Ok(())
}

// moved to tests/common/test_project.rs
