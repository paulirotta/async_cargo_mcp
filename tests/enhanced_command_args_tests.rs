//! Enhanced command argument handling tests
//!
//! Tests for new features:
//! 1. Binary arguments support in cargo run
//! 2. Enhanced argument support in cargo test for test selection
//! 3. Additional options for cargo build

use anyhow::Result;
mod common;
use common::test_project::{
    create_project_with_binary_args, create_project_with_integration_tests,
};
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::env;
use tokio::process::Command;

// moved to tests/common/test_project.rs

async fn call_mcp_tool(tool_name: &str, args: serde_json::Value) -> Result<String> {
    let original_dir = env::current_dir()?;

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .current_dir(&original_dir);
            },
        ))?)
        .await?;

    let arguments = match args {
        serde_json::Value::Object(map) => Some(map),
        _ => None,
    };

    let result = client
        .call_tool(CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments,
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test cargo run with binary arguments
/// This test verifies the new binary_args functionality
#[tokio::test]
async fn test_cargo_run_with_binary_args() {
    let temp_project = create_project_with_binary_args()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    // Test running with binary arguments
    let result = call_mcp_tool(
        "run",
        serde_json::json!({
            "working_directory": project_path,
            "bin_name": "test_binary",
            "binary_args": ["--special", "arg1", "arg2"]
        }),
    )
    .await;

    match result {
        Ok(output) => {
            println!("Run with args test output: {output}");
            assert!(output.contains("SPECIAL_MODE_ACTIVATED"));
            assert!(output.contains("arg[0]: --special"));
            assert!(output.contains("arg[1]: arg1"));
            assert!(output.contains("arg[2]: arg2"));
        }
        Err(e) => {
            panic!("Run with args test failed: {e}");
        }
    }
}

/// Test cargo run with features and release mode
/// This test verifies the new features and release functionality
#[tokio::test]
async fn test_cargo_run_with_features_and_release() {
    let temp_project = create_project_with_binary_args()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = call_mcp_tool(
        "run",
        serde_json::json!({
            "working_directory": project_path,
            "bin_name": "test_binary",
            "release": true
        }),
    )
    .await;

    match result {
        Ok(output) => {
            println!("Run with features/release test output: {output}");
            assert!(output.contains("completed successfully"));
        }
        Err(e) => {
            panic!("Run with features/release test failed: {e}");
        }
    }
}

/// Test cargo test with integration test selection  
/// This test verifies the new args functionality in TestRequest
#[tokio::test]
async fn test_cargo_test_integration_test_selection() {
    let temp_project = create_project_with_integration_tests()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    // Test selecting only integration tests
    let result = call_mcp_tool(
        "test",
        serde_json::json!({
            "working_directory": project_path,
            "args": ["--test", "integration_tests"]
        }),
    )
    .await;

    match result {
        Ok(output) => {
            println!("Integration test selection output: {output}");
            assert!(output.contains("integration_test_add"));
            // Should NOT contain unit tests from src/lib.rs (those have different names)
            assert!(!output.contains("running 1 test")); // Integration tests should run 2 tests
            assert!(output.contains("running 2 tests")); // Should run the 2 integration tests
        }
        Err(e) => {
            panic!("Integration test selection failed: {e}");
        }
    }
}

/// Test cargo test with test name filtering
/// This test should FAIL initially because args are not supported in TestRequest  
#[tokio::test]
#[ignore] // Will be enabled after implementing the feature
async fn test_cargo_test_name_filtering() {
    let temp_project = create_project_with_integration_tests()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    // Test filtering by test name
    let result = call_mcp_tool(
        "test",
        serde_json::json!({
            "working_directory": project_path,
            "test_name": "integration_test_add"
        }),
    )
    .await;

    match result {
        Ok(output) => {
            println!("Test name filtering output: {output}");
            assert!(output.contains("integration_test_add"));
            // Should NOT contain integration_test_multiply
            assert!(!output.contains("integration_test_multiply"));
        }
        Err(e) => {
            panic!("Test name filtering failed: {e}");
        }
    }
}

/// Test cargo build with additional features
/// This test should FAIL initially because these options are not supported yet
#[tokio::test]
// Enabled now that build feature handling is implemented
async fn test_cargo_build_with_features() {
    let temp_project = create_project_with_binary_args()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = call_mcp_tool(
        "build",
        serde_json::json!({
            "working_directory": project_path,
            "features": ["default"],
            "all_features": false,
            "no_default_features": false,
            "release": true,
            "target": "x86_64-unknown-linux-gnu"
        }),
    )
    .await;

    match result {
        Ok(output) => {
            println!("Build with features test output: {output}");
            assert!(output.contains("completed successfully"));
        }
        Err(e) => {
            panic!("Build with features test failed: {e}");
        }
    }
}

/// Test cargo build with workspace options
/// This test should FAIL initially because these options are not supported yet
#[tokio::test]
// Enabled now that workspace/job handling is implemented
async fn test_cargo_build_workspace_options() {
    let temp_project = create_project_with_binary_args()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = call_mcp_tool(
        "build",
        serde_json::json!({
            "working_directory": project_path,
            "workspace": true,
            "exclude": ["excluded_package"],
            "jobs": 4
        }),
    )
    .await;

    match result {
        Ok(output) => {
            println!("Build workspace options test output: {output}");
            assert!(output.contains("completed successfully"));
        }
        Err(e) => {
            panic!("Build workspace options test failed: {e}");
        }
    }
}
