//! Comprehensive tests for async cargo operations
//! This file consolidates all async cargo command tests from individual files

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

/// Create a basic Cargo project in a temporary directory for testing
async fn create_basic_project() -> Result<TempDir> {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo_mcp_test_")
        .rand_bytes(6)
        .tempdir()?;

    let project_path = temp_dir.path();

    // Create Cargo.toml
    fs::write(
        project_path.join("Cargo.toml"),
        r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .await?;

    // Create src directory
    fs::create_dir_all(project_path.join("src")).await?;

    // Create main.rs
    fs::write(
        project_path.join("src").join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    )
    .await?;

    Ok(temp_dir)
}
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

/// Test async build operation
#[tokio::test]
async fn test_async_build() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
            },
        ))?)
        .await?;

    // Start async build
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
    let operation_id = extract_operation_id(&build_text).expect("operation id should be present");

    // Wait for completion
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [operation_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Verify successful completion
    assert!(
        wait_text.contains("OPERATION COMPLETED"),
        "Build should complete successfully"
    );

    Ok(())
}

/// Test async check operation  
#[tokio::test]
async fn test_async_check() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
            },
        ))?)
        .await?;

    // Start async check
    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let check_text = format!("{:?}", check_result.content);
    let operation_id = extract_operation_id(&check_text).expect("operation id should be present");

    // Wait for completion
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [operation_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Verify successful completion
    assert!(
        wait_text.contains("OPERATION COMPLETED"),
        "Check should complete successfully"
    );

    Ok(())
}

/// Test multiple async operations completed together  
#[tokio::test]
async fn test_multiple_async_operations() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
            },
        ))?)
        .await?;

    // Start multiple async operations
    let build_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path.clone(),
                "enable_async_notification": true
            })),
        })
        .await?;

    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    // Extract operation IDs
    let build_text = format!("{:?}", build_result.content);
    let check_text = format!("{:?}", check_result.content);

    let build_id = extract_operation_id(&build_text).expect("build operation id should be present");
    let check_id = extract_operation_id(&check_text).expect("check operation id should be present");

    // Wait for both operations
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [build_id, check_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Verify both operations completed
    assert!(
        wait_text.matches("OPERATION COMPLETED").count() >= 2,
        "Both operations should complete successfully"
    );

    Ok(())
}
