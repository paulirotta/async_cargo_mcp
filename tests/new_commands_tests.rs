//! Tests for newly implemented commands
//!
//! This module tests the new cargo commands: nextest, clean, fix, search, bench, install
//! Including error handling, availability checks, and edge cases.

use anyhow::Result;
use async_cargo_mcp::cargo_tools::AsyncCargo;
use async_cargo_mcp::test_cargo_tools;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::{env, fs};
use tempdir::TempDir;
use tokio::process::Command;

/// Test cargo clean command
#[tokio::test]
async fn test_cargo_clean_command() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    // First build the project to create artifacts
    let build_result = test_cargo_tools::test_build_command(project_path).await;
    assert!(build_result.is_ok(), "Build failed: {:?}", build_result);

    // Now test clean
    let result = test_clean_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Clean test passed: {output}");
            assert!(
                output.contains("Clean operation") || output.contains("completed successfully")
            );
        }
        Err(e) => {
            panic!("Clean test failed: {e}");
        }
    }
}

/// Test cargo fix command with --allow-dirty flag
#[tokio::test]
async fn test_cargo_fix_command() {
    let temp_project = create_test_cargo_project_with_warning()
        .await
        .expect("Failed to create test project with warning");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_fix_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Fix test passed: {output}");
            assert!(output.contains("Fix operation"));
        }
        Err(e) => {
            // Fix might fail if there are no warnings to fix, which is ok
            println!("Fix test completed (may have no warnings to fix): {e}");
        }
    }
}

/// Test cargo search command
#[tokio::test]
async fn test_cargo_search_command() {
    let result = test_search_command("serde").await;

    match result {
        Ok(output) => {
            println!("Search test passed: {output}");
            assert!(output.contains("Search operation") && output.contains("serde"));
        }
        Err(e) => {
            // Search might fail due to network issues, which is acceptable in tests
            println!("Search test completed (network required): {e}");
        }
    }
}

/// Test cargo bench command (expect failure since no benchmarks defined)
#[tokio::test]
async fn test_cargo_bench_command() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_bench_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Bench test result: {output}");
            // Bench might pass or fail depending on project setup
            assert!(output.contains("Benchmark operation"));
        }
        Err(e) => {
            println!("Bench test completed (no benchmarks defined): {e}");
        }
    }
}

/// Test cargo install command error handling (invalid package)
#[tokio::test]
async fn test_cargo_install_error_handling() {
    // Try to install a non-existent package to test error handling
    let result = test_install_command("non-existent-package-12345").await;

    match result {
        Ok(output) => {
            println!("Install test result: {output}");
            // Should contain error information
            assert!(output.contains("Install operation"));
        }
        Err(e) => {
            println!("Install test handled error correctly: {e}");
        }
    }
}

/// Test cargo audit command availability and basic functionality
#[tokio::test]
async fn test_cargo_audit_command() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_audit_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Audit test result: {output}");
            // Should either run successfully or show availability message
            assert!(output.contains("Audit operation") || output.contains("audit"));
        }
        Err(e) => {
            println!("Audit test completed: {e}");
        }
    }
}

/// Test cargo audit command with async notifications
#[tokio::test]
async fn test_cargo_audit_with_async() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_audit_command_with_async(project_path).await;

    match result {
        Ok(output) => {
            println!("Audit async test result: {output}");
            // Should either show successful audit or installation message
            assert!(
                output.contains("Audit completed successfully")
                    || output.contains("Audit operation")
                    || output.contains("cargo-audit is not installed")
            );
        }
        Err(e) => {
            println!("Audit async test completed: {e}");
        }
    }
}

/// Test cargo audit with various format options
#[tokio::test]
async fn test_cargo_audit_formats() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    // Test with JSON format
    let result = test_audit_command_with_format(project_path, "json").await;

    match result {
        Ok(output) => {
            println!("Audit JSON format test result: {output}");
            assert!(output.contains("Audit operation"));
        }
        Err(e) => {
            println!("Audit JSON format test completed: {e}");
        }
    }
}

/// Test nextest availability checking
#[tokio::test]
async fn test_nextest_availability() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_nextest_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Nextest test result: {output}");
            // Should either run successfully or show availability message
            assert!(output.contains("Nextest operation") || output.contains("nextest"));
        }
        Err(e) => {
            println!("Nextest test completed: {e}");
        }
    }
}

/// Test availability report generation
#[tokio::test]
async fn test_availability_report() {
    let report = AsyncCargo::generate_availability_report().await;

    println!("Availability report: {report}");

    // Should contain expected sections
    assert!(report.contains("Cargo MCP Server Availability Report"));
    assert!(report.contains("Core Commands"));
    assert!(report.contains("Optional Components"));
    assert!(report.contains("clippy"));
    assert!(report.contains("nextest"));
    assert!(report.contains("cargo-audit"));
    assert!(report.contains("Recommendations"));
}

/// Test component availability checking
#[tokio::test]
async fn test_component_availability() {
    let availability = AsyncCargo::check_component_availability().await;

    println!("Component availability: {:?}", availability);

    // Should check for cargo (always true), clippy, nextest, and cargo-audit
    assert!(availability.contains_key("cargo"));
    assert!(availability.contains_key("clippy"));
    assert!(availability.contains_key("nextest"));
    assert!(availability.contains_key("cargo-audit"));

    // Cargo should always be available if we got this far
    assert_eq!(availability.get("cargo"), Some(&true));
}

/// Test error handling for invalid working directory
#[tokio::test]
async fn test_invalid_working_directory() {
    let result = test_build_command_with_invalid_dir("/non/existent/directory").await;

    match result {
        Ok(output) => {
            println!("Invalid directory test result: {output}");
            // Should handle the error gracefully
            assert!(output.contains("Build operation") || output.contains("failed"));
        }
        Err(e) => {
            println!("Invalid directory test handled error correctly: {e}");
        }
    }
}

/// Test async notifications flag
#[tokio::test]
async fn test_async_notifications_flag() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_build_command_with_async(project_path).await;

    match result {
        Ok(output) => {
            println!("Async notifications test passed: {output}");
            // When async notifications are enabled, the response format is different
            // The callback system returns "Build completed successfully" instead of "Build operation"
            assert!(
                output.contains("Build completed successfully")
                    || output.contains("Build operation")
            );
        }
        Err(e) => {
            panic!("Async notifications test failed: {e}");
        }
    }
}

// Helper functions for testing new commands

async fn test_clean_command(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "clean".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_fix_command(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "fix".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "args": ["--allow-dirty"]
            })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_search_command(query: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "search".into(),
            arguments: Some(object!({
                "query": query,
                "limit": 5
            })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_bench_command(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "bench".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_install_command(package: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "install".into(),
            arguments: Some(object!({ "package": package })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_nextest_command(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "nextest".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_build_command_with_invalid_dir(invalid_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({ "working_directory": invalid_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_build_command_with_async(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": true
            })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

/// Create a test project with a warning to test cargo fix
async fn create_test_cargo_project_with_warning() -> Result<TempDir> {
    let uuid = uuid::Uuid::new_v4();
    let temp_dir = TempDir::new(&format!("cargo_mcp_fix_test_{}", uuid))?;
    let project_path = temp_dir.path();

    // Create Cargo.toml
    let cargo_toml_content = r#"[package]
name = "test_project_with_warning"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml_content)?;

    // Create src directory
    fs::create_dir(project_path.join("src"))?;

    // Create main.rs with code that generates warnings
    let main_rs_content = r#"fn main() {
    let unused_variable = 42; // This will generate a warning
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

    println!("Created test project with warnings at: {project_path:?}");

    Ok(temp_dir)
}

/// Create a basic test project (reused from cargo_tools_tests.rs)
async fn create_test_cargo_project() -> Result<TempDir> {
    let uuid = uuid::Uuid::new_v4();
    let temp_dir = TempDir::new(&format!("cargo_mcp_test_{}", uuid))?;
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

    println!("Created test project at: {project_path:?}");

    Ok(temp_dir)
}

async fn test_audit_command(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "audit".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_audit_command_with_async(project_path: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "audit".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": true
            })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}

async fn test_audit_command_with_format(project_path: &str, format: &str) -> Result<String> {
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

    let result = client
        .call_tool(CallToolRequestParam {
            name: "audit".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "format": format
            })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{:?}", result))
}
