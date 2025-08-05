//! Cargo functionality integration tests
//!
//! These tests create temporary cargo projects and test the cargo tools

use anyhow::Result;
use async_cargo_mcp::test_cargo_tools;
use std::fs;
use tempfile::TempDir;

/// Test cargo build in a temporary project
#[tokio::test]
async fn test_cargo_build_in_temp_project() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_cargo_tools::test_build_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Build test passed: {output}");
            assert!(
                output.contains("Build operation") || output.contains("completed successfully")
            );
        }
        Err(e) => {
            panic!("Build test failed: {e}");
        }
    }
}

/// Test cargo check in a temporary project  
#[tokio::test]
async fn test_cargo_check_in_temp_project() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_cargo_tools::test_check_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Check test passed: {output}");
            assert!(
                output.contains("Check operation") || output.contains("completed successfully")
            );
        }
        Err(e) => {
            panic!("Check test failed: {e}");
        }
    }
}

/// Test cargo add dependency in a temporary project
#[tokio::test]
async fn test_cargo_add_dependency() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_cargo_tools::test_add_dependency(project_path, "serde", Some("1.0")).await;

    match result {
        Ok(output) => {
            println!("Add dependency test passed: {output}");
            assert!(output.contains("Add operation") && output.contains("serde"));
        }
        Err(e) => {
            panic!("Add dependency test failed: {e}");
        }
    }
}

/// Test cargo remove dependency in a temporary project
#[tokio::test]
async fn test_cargo_remove_dependency() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    // First add a dependency
    let _add_result = test_cargo_tools::test_add_dependency(project_path, "serde", Some("1.0"))
        .await
        .expect("Failed to add dependency for removal test");

    // Then remove it
    let result = test_cargo_tools::test_remove_dependency(project_path, "serde").await;

    match result {
        Ok(output) => {
            println!("Remove dependency test passed: {output}");
            assert!(output.contains("Remove operation") && output.contains("serde"));
        }
        Err(e) => {
            panic!("Remove dependency test failed: {e}");
        }
    }
}

/// Test cargo test in a temporary project
#[tokio::test]
async fn test_cargo_test_in_temp_project() {
    let temp_project = create_test_cargo_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_cargo_tools::test_test_command(project_path).await;

    match result {
        Ok(output) => {
            println!("Test command test passed: {output}");
            assert!(output.contains("Test operation") || output.contains("completed successfully"));
        }
        Err(e) => {
            panic!("Test command test failed: {e}");
        }
    }
}

/// Create a temporary cargo project for testing
///
/// This function creates a minimal but valid Rust project in a temporary directory
/// that can be used for testing cargo commands safely.
async fn create_test_cargo_project() -> Result<TempDir> {
    let uuid = uuid::Uuid::new_v4();
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("cargo_mcp_test_{}_", uuid))
        .tempdir()?;
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
