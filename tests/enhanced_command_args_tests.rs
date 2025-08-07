//! Enhanced command argument handling tests
//!
//! Tests for new features:
//! 1. Binary arguments support in cargo run
//! 2. Enhanced argument support in cargo test for test selection
//! 3. Additional options for cargo build

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::env;
use std::fs;
use tempfile::TempDir;
use tokio::process::Command;

/// Create a test cargo project with a binary that accepts arguments
async fn create_test_project_with_binary_args() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let cargo_toml = r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test_binary"
path = "src/bin/test_binary.rs"
"#;

    let main_rs = r#"
fn main() {
    println!("Hello from main!");
}
"#;

    let test_binary_rs = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("test_binary called with {} args:", args.len() - 1);
    for (i, arg) in args.iter().skip(1).enumerate() {
        println!("  arg[{}]: {}", i, arg);
    }
    
    if args.len() > 1 && args[1] == "--special" {
        println!("SPECIAL_MODE_ACTIVATED");
    }
}
"#;

    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)?;

    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("main.rs"), main_rs)?;

    let bin_dir = src_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
    fs::write(bin_dir.join("test_binary.rs"), test_binary_rs)?;

    Ok(temp_dir)
}

/// Create a test cargo project with integration tests
async fn create_test_project_with_integration_tests() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let cargo_toml = r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;

    let main_rs = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("Hello, world!");
}
"#;

    let unit_test = r#"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#;

    let integration_test = r#"
use test_project::add;

#[test]
fn integration_test_add() {
    assert_eq!(add(10, 20), 30);
}

#[test]
fn integration_test_multiply() {
    // This would fail but demonstrates test selection
    assert_eq!(add(2, 3), 5); // This is still add, not multiply
}
"#;

    fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)?;

    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;

    let main_rs_with_tests = format!("{main_rs}\n\n{unit_test}");
    fs::write(src_dir.join("lib.rs"), main_rs_with_tests)?;
    fs::write(
        src_dir.join("main.rs"),
        "fn main() { println!(\"Hello, world!\"); }",
    )?;

    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir)?;
    fs::write(tests_dir.join("integration_tests.rs"), integration_test)?;

    Ok(temp_dir)
}

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
    let temp_project = create_test_project_with_binary_args()
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
    let temp_project = create_test_project_with_binary_args()
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
    let temp_project = create_test_project_with_integration_tests()
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
    let temp_project = create_test_project_with_integration_tests()
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
#[ignore] // Will be enabled after implementing the feature
async fn test_cargo_build_with_features() {
    let temp_project = create_test_project_with_binary_args()
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
#[ignore] // Will be enabled after implementing the feature
async fn test_cargo_build_workspace_options() {
    let temp_project = create_test_project_with_binary_args()
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
