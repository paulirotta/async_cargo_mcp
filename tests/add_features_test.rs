//! Test the enhanced add command with features support

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::env;
use std::path::PathBuf;
use tokio::{fs, process::Command};

mod common;
use common::test_project::create_basic_project;

/// Helper to build path to the already-built async_cargo_mcp binary.
fn server_binary(original_dir: &std::path::Path) -> PathBuf {
    original_dir
        .join("target")
        .join("debug")
        .join("async_cargo_mcp")
}

#[tokio::test]
async fn test_add_with_features() -> Result<()> {
    let temp_project = create_basic_project().await?;
    let project_path = temp_project.path().to_str().unwrap();
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    // Test adding serde with derive feature
    let result = client
        .call_tool(CallToolRequestParam {
            name: "add".into(),
            arguments: Some(object!({
                "name": "serde",
                "version": "1.0",
                "features": ["derive"],
                "working_directory": project_path
            })),
        })
        .await?;

    println!("Add with features result: {:?}", result);

    client.cancel().await?;

    // Verify the dependency was added correctly by reading Cargo.toml
    let cargo_toml_path = temp_project.path().join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path).await?;

    println!("Cargo.toml content: {}", cargo_toml_content);

    // Check if serde was added with the derive feature
    assert!(cargo_toml_content.contains("serde"));
    assert!(cargo_toml_content.contains("derive"));

    Ok(())
}

#[tokio::test]
async fn test_add_with_no_default_features() -> Result<()> {
    let temp_project = create_basic_project().await?;
    let project_path = temp_project.path().to_str().unwrap();
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    // Test adding tokio with no default features
    let result = client
        .call_tool(CallToolRequestParam {
            name: "add".into(),
            arguments: Some(object!({
                "name": "tokio",
                "features": ["rt"],
                "no_default_features": true,
                "working_directory": project_path
            })),
        })
        .await?;

    println!("Add with no default features result: {:?}", result);

    client.cancel().await?;

    // Verify the dependency was added with no default features
    let cargo_toml_path = temp_project.path().join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path).await?;

    println!("Cargo.toml content: {}", cargo_toml_content);

    // Check if tokio was added with no default features
    assert!(cargo_toml_content.contains("tokio"));
    assert!(cargo_toml_content.contains("default-features = false"));

    Ok(())
}
