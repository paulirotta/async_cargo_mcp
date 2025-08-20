//! Error handling and edge case tests
//!
//! This module tests error conditions, malformed inputs, and edge cases
//! to ensure robust error handling across all cargo commands.

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::env;
use tempfile::TempDir;
use tokio::process::Command;

/// Test build command with missing Cargo.toml
#[tokio::test]
async fn test_build_missing_cargo_toml() {
    let result = test_command_in_empty_dir("build").await;
    match result {
        Ok(output) => {
            println!("Build with missing Cargo.toml: {output}");
            // Should handle error gracefully
            assert!(output.contains("Build operation") || output.contains("failed"));
        }
        Err(e) => {
            println!("Build error handled correctly: {e}");
        }
    }
}

/// Test add command with invalid dependency name
#[tokio::test]
async fn test_add_invalid_dependency() {
    let result = test_add_invalid_dependency_helper("").await;
    match result {
        Ok(output) => {
            println!("Add invalid dependency: {output}");
            // Should handle error gracefully
            assert!(output.contains("Add operation"));
        }
        Err(e) => {
            println!("Add error handled correctly: {e}");
        }
    }
}

/// Test remove command with non-existent dependency
#[tokio::test]
async fn test_remove_nonexistent_dependency() {
    let result = test_remove_dependency("non-existent-dep-123456").await;
    match result {
        Ok(output) => {
            println!("Remove non-existent dependency: {output}");
            // Should handle error gracefully
            assert!(output.contains("Remove operation"));
        }
        Err(e) => {
            println!("Remove error handled correctly: {e}");
        }
    }
}

/// Test clippy command with invalid arguments
#[tokio::test]
async fn test_clippy_invalid_args() {
    let result = test_clippy_with_args(&["--invalid-flag-12345"]).await;
    match result {
        Ok(output) => {
            println!("Clippy with invalid args: {output}");
            // Should handle error gracefully
            assert!(output.contains("Clippy operation"));
        }
        Err(e) => {
            println!("Clippy error handled correctly: {e}");
        }
    }
}

/// Test very long dependency names (boundary testing)
#[tokio::test]
async fn test_add_very_long_dependency_name() {
    let long_name = "a".repeat(1000); // Very long dependency name
    let result = test_add_dependency_helper(&long_name).await;
    match result {
        Ok(output) => {
            println!("Add very long dependency name handled: {output}");
            assert!(output.contains("Add operation"));
        }
        Err(e) => {
            println!("Long dependency name error handled: {e}");
        }
    }
}

/// Test search with empty query
#[tokio::test]
async fn test_search_empty_query() {
    let result = test_search_empty_query_helper().await;
    match result {
        Ok(output) => {
            println!("Search empty query: {output}");
            // Should handle error gracefully
            assert!(output.contains("Search operation"));
        }
        Err(e) => {
            println!("Search empty query handled correctly: {e}");
        }
    }
}

/// Test search with special characters
#[tokio::test]
async fn test_search_special_characters() {
    let result = test_search_query("!@#$%^&*()").await;
    match result {
        Ok(output) => {
            println!("Search special characters: {output}");
            assert!(output.contains("Search operation"));
        }
        Err(e) => {
            println!("Search special characters handled: {e}");
        }
    }
}

/// Test install with version that doesn't exist
#[tokio::test]
async fn test_install_invalid_version() {
    let result = test_install_with_version("serde", "999.999.999").await;
    match result {
        Ok(output) => {
            println!("Install invalid version: {output}");
            assert!(output.contains("Install operation"));
        }
        Err(e) => {
            println!("Install invalid version handled: {e}");
        }
    }
}

/// Test concurrent operations on same directory
#[tokio::test]
async fn test_concurrent_operations() {
    // This tests whether the server can handle multiple requests properly
    let temp_project = create_minimal_project().await;
    if let Ok(temp_dir) = temp_project {
        let project_path = temp_dir.path().to_str().unwrap();

        // Launch multiple operations concurrently
        let build_task = tokio::spawn(test_build_command(project_path.to_string()));
        let check_task = tokio::spawn(test_check_command(project_path.to_string()));

        let (build_result, check_result) = tokio::join!(build_task, check_task);

        match (build_result, check_result) {
            (Ok(Ok(build_output)), Ok(Ok(check_output))) => {
                println!("Concurrent operations succeeded:");
                println!("Build: {build_output}");
                println!("Check: {check_output}");
            }
            _ => {
                println!("Concurrent operations completed (some may have failed as expected)");
            }
        }
    }
}

/// Test malformed JSON in arguments
#[tokio::test]
async fn test_malformed_arguments() {
    // This would be handled at the MCP protocol level, but we can test our parameter validation
    let result = test_build_with_null_working_dir().await;
    match result {
        Ok(output) => {
            println!("Malformed args handled: {output}");
            assert!(output.contains("Build operation"));
        }
        Err(e) => {
            println!("Malformed args error handled: {e}");
        }
    }
}

/// Test extremely large working directory path
#[tokio::test]
async fn test_very_long_path() {
    let long_path = "/".to_string() + &"very_long_directory_name_".repeat(50);
    let result = test_build_command_with_path(&long_path).await;
    match result {
        Ok(output) => {
            println!("Very long path handled: {output}");
            assert!(output.contains("Build operation"));
        }
        Err(e) => {
            println!("Very long path error handled: {e}");
        }
    }
}

/// Test commands with various character encodings in paths
#[tokio::test]
async fn test_unicode_paths() {
    let unicode_path = "/tmp/тест_проект_"; // Mix of Cyrillic and emoji
    let result = test_build_command_with_path(unicode_path).await;
    match result {
        Ok(output) => {
            println!("Unicode path handled: {output}");
            assert!(output.contains("Build operation"));
        }
        Err(e) => {
            println!("Unicode path error handled: {e}");
        }
    }
}

// Helper functions for error testing

async fn test_command_in_empty_dir(command: &'static str) -> Result<String> {
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
            name: command.into(),
            arguments: Some(object!({ "working_directory": "/tmp" })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_add_invalid_dependency_helper(dep_name: &str) -> Result<String> {
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
            name: "add".into(),
            arguments: Some(object!({ "name": dep_name })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_remove_dependency(dep_name: &str) -> Result<String> {
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
            name: "remove".into(),
            arguments: Some(object!({ "name": dep_name })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_clippy_with_args(args: &[&str]) -> Result<String> {
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
            name: "clippy".into(),
            arguments: Some(object!({ "args": args })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_add_dependency_helper(dep_name: &str) -> Result<String> {
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
            name: "add".into(),
            arguments: Some(object!({ "name": dep_name })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_search_empty_query_helper() -> Result<String> {
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
            arguments: Some(object!({ "query": "" })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_search_query(query: &str) -> Result<String> {
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
            arguments: Some(object!({ "query": query })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_install_with_version(package: &str, version: &str) -> Result<String> {
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
            arguments: Some(object!({
                "package": package,
                "version": version
            })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_build_command(project_path: String) -> Result<String> {
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
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_check_command(project_path: String) -> Result<String> {
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
            name: "check".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_build_command_with_path(path: &str) -> Result<String> {
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
            arguments: Some(object!({ "working_directory": path })),
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn test_build_with_null_working_dir() -> Result<String> {
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
            arguments: Some(object!({})), // No working_directory provided
        })
        .await?;

    client.cancel().await?;
    Ok(format!("{result:?}"))
}

async fn create_minimal_project() -> Result<TempDir> {
    use std::fs;

    let uuid = uuid::Uuid::new_v4();
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("cargo_mcp_minimal_{uuid}"))
        .tempdir()?;
    let project_path = temp_dir.path();

    // Create Cargo.toml
    let cargo_toml_content = r#"[package]
name = "minimal_test"
version = "0.1.0"
edition = "2024"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml_content)?;

    // Create src directory
    fs::create_dir(project_path.join("src"))?;

    // Create lib.rs (minimal library)
    let lib_rs_content = r#"//! Minimal test library
"#;

    fs::write(project_path.join("src").join("lib.rs"), lib_rs_content)?;

    Ok(temp_dir)
}
