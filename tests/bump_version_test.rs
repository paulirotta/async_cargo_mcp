//! Test to verify bump_version tool functionality
//! This test ensures that the bump_version tool correctly handles version bumping with cargo-edit

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::fs;
use tempfile::TempDir;
use tokio::process::Command;

/// Create a basic Rust project in a temporary directory
async fn create_basic_project() -> Result<TempDir> {
    let temp = TempDir::new()?;
    let temp_path = temp.path();

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    fs::write(temp_path.join("Cargo.toml"), cargo_toml)?;

    // Create src directory and main.rs
    let src_dir = temp_path.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        src_dir.join("main.rs"),
        "fn main() { println!(\"Hello, world!\"); }",
    )?;

    Ok(temp)
}

#[tokio::test]
async fn test_bump_version_patch_success() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "bump_version".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "bump_type": "patch"
            })),
        })
        .await?;

    let response_text = format!("{:?}", result.content);
    println!("Response for bump_version patch: {}", response_text);

    // Should be synchronous and return immediate result
    assert!(
        !response_text.contains("started at"),
        "bump_version should be synchronous but returned async response: {}",
        response_text
    );

    // Should either succeed or indicate cargo-edit is missing
    assert!(
        response_text.contains("completed successfully")
            || response_text.contains("cargo-edit")
            || response_text.contains("not installed"),
        "bump_version should return appropriate result: {}",
        response_text
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_bump_version_minor_success() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "bump_version".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "bump_type": "minor"
            })),
        })
        .await?;

    let response_text = format!("{:?}", result.content);
    println!("Response for bump_version minor: {}", response_text);

    // Should be synchronous and return immediate result
    assert!(
        !response_text.contains("started at"),
        "bump_version should be synchronous: {}",
        response_text
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_bump_version_major_success() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "bump_version".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "bump_type": "major"
            })),
        })
        .await?;

    let response_text = format!("{:?}", result.content);
    println!("Response for bump_version major: {}", response_text);

    // Should be synchronous and return immediate result
    assert!(
        !response_text.contains("started at"),
        "bump_version should be synchronous: {}",
        response_text
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_bump_version_invalid_type() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "bump_version".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "bump_type": "invalid"
            })),
        })
        .await?;

    let response_text = format!("{:?}", result.content);
    println!("Response for bump_version invalid: {}", response_text);

    // Should return error for invalid bump type
    assert!(
        response_text.contains("invalid")
            || response_text.contains("error")
            || response_text.contains("failed"),
        "bump_version should return error for invalid bump type: {}",
        response_text
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_bump_version_with_dry_run() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "bump_version".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "bump_type": "patch",
                "dry_run": true
            })),
        })
        .await?;

    let response_text = format!("{:?}", result.content);
    println!("Response for bump_version dry-run: {}", response_text);

    // Should indicate dry run in the response
    if !response_text.contains("not installed") {
        assert!(
            response_text.contains("dry run") || response_text.contains("dry-run"),
            "bump_version should indicate dry run mode: {}",
            response_text
        );
    }

    client.cancel().await?;
    Ok(())
}
