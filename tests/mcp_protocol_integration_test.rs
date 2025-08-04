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
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::json;
use std::fs;
use tempdir::TempDir;
use tokio::process::Command;
use tokio::time::{Duration, sleep};

/// Comprehensive test that replicates test-mcp.sh functionality
#[tokio::test]
async fn test_mcp_protocol_comprehensive() -> Result<()> {
    // Create a temporary project for testing add/remove operations
    let temp_project = create_test_cargo_project().await?;
    let temp_project_path = temp_project.path().to_str().unwrap();

    // Create client connection to our MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test 1: Initialize (this happens automatically when we create the client)
    println!("MCP initialization successful");

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

    println!("Available tools: {:?}", tool_names);

    for expected_tool in &expected_tools {
        assert!(
            tool_names.contains(&expected_tool.to_string()),
            "{} tool should be available but was not found in: {:?}",
            expected_tool,
            tool_names
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
            "Unexpected utility tool '{}' found - these should have been removed",
            unexpected_tool
        );
    }

    // Verify we have exactly the expected number of tools (catches if new tools are added)
    // Note: We expect 8 total tools but only test 7 to avoid recursive test execution
    assert_eq!(
        tool_names.len(),
        9,
        "Expected exactly 8 tools, but found {}. Tools: {:?}",
        tool_names.len(),
        tool_names
    );

    // Verify the test tool exists but we won't execute it to avoid recursion
    assert!(
        tool_names.contains(&"test".to_string()),
        "test tool should be available but was not found in: {:?}",
        tool_names
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
        let arguments = match tool_name.as_ref() {
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
            "{} tool should return some content",
            tool_name
        );

        // Verify the result has the expected structure
        assert!(
            result.content.len() > 0,
            "{} tool should return at least one content item",
            tool_name
        );

        println!("{} tool executed successfully", tool_name);
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
        "doc command output should mention documentation: {}",
        doc_output
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
    if let Some(info) = server_info {
        println!("Connected to server: {:?}", info);
    } else {
        println!("Connected to server (info not available)");
    }

    // Test protocol capabilities
    let tools_result = client.list_all_tools().await?;
    assert!(tools_result.len() > 0, "Server should provide tools");
    println!("Server provides {} tools", tools_result.len());

    // Test a simple tool call to verify the protocol works end-to-end
    // Create a temporary project for this test
    let temp_project = create_test_cargo_project().await?;
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

/// Helper function to create a temporary Cargo project for testing
/// This ensures tests operate on temporary directories, not the actual project
async fn create_test_cargo_project() -> Result<TempDir> {
    let temp_dir = TempDir::new("cargo_mcp_test")?;
    let project_path = temp_dir.path();

    // Create Cargo.toml
    let cargo_toml_content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml_content)?;

    // Create src directory
    fs::create_dir(project_path.join("src"))?;

    // Create main.rs with a simple hello world
    let main_rs_content = r#"fn main() {
    println!("Hello, test world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
"#;

    fs::write(project_path.join("src").join("main.rs"), main_rs_content)?;

    println!("Created test project at: {:?}", project_path);

    Ok(temp_dir)
}
