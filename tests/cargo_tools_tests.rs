//! Cargo functionality integration tests
//!
//! These tests create temporary cargo projects and test the cargo tools

use async_cargo_mcp::test_cargo_tools;
mod common;
use common::test_project::create_basic_project;

/// Test cargo build in a temporary project
#[tokio::test]
async fn test_cargo_build_in_temp_project() {
    let temp_project = create_basic_project()
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
    let temp_project = create_basic_project()
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
    let temp_project = create_basic_project()
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
    let temp_project = create_basic_project()
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
    let temp_project = create_basic_project()
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

// moved to tests/common/test_project.rs

/// Test cargo doc in a temporary project
#[tokio::test]
async fn test_cargo_doc_generates() {
    let temp_project = create_basic_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    let result = test_cargo_tools::test_doc_command_with_content(project_path).await;

    match result {
        Ok(output) => {
            println!("Doc test output: {output}");
            assert!(output.contains("Documentation generated at:"));
        }
        Err(e) => {
            panic!("Doc test failed: {e}");
        }
    }
}
