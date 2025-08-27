//! Comprehensive tests for wait behavior and operation completion
//! This file consolidates wait-related tests from individual files

#[path = "common/mod.rs"]
mod common;
use anyhow::Result;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::time::Instant;
use tokio::process::Command;
/// Extract operation ID from tool response text
fn extract_operation_id(s: &str) -> Option<String> {
    let lines: Vec<&str> = s.lines().collect();
    for line in lines {
        // Look for patterns like "Build operation op_build_0 started" or "Operation ID: op_xxx"
        if line.contains("operation op_") {
            // Find "op_" and extract the operation ID
            if let Some(start) = line.find("op_") {
                let id_part = &line[start..];
                if let Some(end) = id_part.find(' ') {
                    return Some(id_part[..end].to_string());
                } else if let Some(end) = id_part.find(')') {
                    return Some(id_part[..end].to_string());
                } else {
                    // If no space or closing paren, take the whole remaining part
                    return Some(id_part.to_string());
                }
            }
        }
        // Also check for the "Operation ID:" pattern
        else if line.contains("Operation ID:")
            && let Some(start) = line.find("Operation ID: ")
        {
            let id_part = &line[start + "Operation ID: ".len()..];
            if let Some(end) = id_part.find(' ') {
                return Some(id_part[..end].to_string());
            } else {
                return Some(id_part.to_string());
            }
        }
    }
    None
}

/// Test waiting for already completed operations returns cached results
#[tokio::test]
async fn test_wait_after_completion() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server with wait tool enabled
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    // Start async operation
    let build_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let build_text = format!("{:?}", build_result.content);
    let op_id = extract_operation_id(&build_text).expect("operation id should be present");

    // Wait for completion (first time)
    let first_wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
        })
        .await?;

    let first_wait_text = format!("{:?}", first_wait_result.content);
    assert!(
        first_wait_text.contains("OPERATION COMPLETED")
            || first_wait_text.contains("OPERATION FAILED")
    );

    // Wait for the SAME operation again (should return cached results immediately)
    let start_time = Instant::now();
    let second_wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
        })
        .await?;
    let elapsed = start_time.elapsed();

    let second_wait_text = format!("{:?}", second_wait_result.content);

    // Should return immediately (within 1 second)
    assert!(
        elapsed.as_secs() < 1,
        "Second wait should return cached results immediately, took {:?}",
        elapsed
    );

    // Should contain the same completion status
    assert!(
        second_wait_text.contains("OPERATION COMPLETED")
            || second_wait_text.contains("OPERATION FAILED")
    );

    Ok(())
}

/// Test waiting for non-existent operation IDs
#[tokio::test]
async fn test_wait_nonexistent_operation() -> Result<()> {
    let temp = create_basic_project().await?;
    let _project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server with wait tool enabled
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    // Try to wait for a non-existent operation ID
    let start_time = Instant::now();
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": ["op_nonexistent_12345"] })),
        })
        .await?;
    let elapsed = start_time.elapsed();

    let wait_text = format!("{:?}", wait_result.content);

    // Should return immediately (within a few milliseconds)
    assert!(
        elapsed.as_millis() < 1000,
        "Wait for non-existent operation should return immediately, but took {:?}",
        elapsed
    );

    // Should indicate that operation was not found
    assert!(
        wait_text.contains("No operation found")
            || wait_text.contains("not found")
            || wait_text.contains("op_nonexistent_12345"),
        "Should indicate operation not found, got: {}",
        wait_text
    );

    Ok(())
}

/// Test waiting for empty operation list (should fail validation)
#[tokio::test]
async fn test_wait_empty_operation_list() -> Result<()> {
    let temp = create_basic_project().await?;
    let _project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server with wait tool enabled
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    // Try to wait for empty operation list - should fail immediately
    let start_time = Instant::now();
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [] })),
        })
        .await;
    let elapsed = start_time.elapsed();

    // Should fail immediately with validation error
    assert!(
        elapsed.as_secs() < 1,
        "Empty operation list validation should fail immediately, but took {:?}",
        elapsed
    );

    // Should return error
    assert!(
        wait_result.is_err(),
        "Wait with empty operation_ids should fail validation"
    );

    Ok(())
}

/// Test waiting for mix of real and fake operation IDs
#[tokio::test]
async fn test_wait_mixed_real_and_fake_operations() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server with wait tool enabled
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    // Start a real operation
    let build_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let first_text = format!("{:?}", build_result.content);
    let real_op_id = extract_operation_id(&first_text).expect("operation id should be present");

    // Wait for both a real operation and a fake one
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": [real_op_id.clone(), "op_fake_999"]
            })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Should get results for both operations
    // One should be completed/failed (the real one)
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));

    // The other should be a helpful message about the missing operation
    assert!(wait_text.contains("No operation found") && wait_text.contains("op_fake_999"));

    // Should contain both operation IDs
    assert!(wait_text.contains(&real_op_id));
    assert!(wait_text.contains("op_fake_999"));

    Ok(())
}
