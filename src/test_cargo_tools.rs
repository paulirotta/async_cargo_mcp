//! Test utilities for cargo functionality
//!
//! This module provides utilities for testing cargo commands in isolation
//! using temporary directories and the working_directory parameter.

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::env;
use std::path::PathBuf;
use tokio::process::Command;

/// Helper to build path to the already-built async_cargo_mcp binary.
fn server_binary(original_dir: &std::path::Path) -> PathBuf {
    // During `cargo test` the binary should already be built; if not, user can rerun.
    original_dir
        .join("target")
        .join("debug")
        .join("async_cargo_mcp")
}

/// Test the build command in a specific directory using working_directory parameter
pub async fn test_build_command(project_path: &str) -> Result<String> {
    let original_dir = env::current_dir()?;

    // Start the MCP client from the original directory
    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    // Use working_directory parameter to specify where cargo build should run
    let result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    eprintln!("TEST_BUILD_COMMAND raw result: {:?}", result);
    let rmcp::model::CallToolResult { content, .. } = &result;
    for (i, c) in content.iter().enumerate() {
        eprintln!(" content[{i}] = {:?}", c);
    }

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test the check command in a specific directory using working_directory parameter
pub async fn test_check_command(project_path: &str) -> Result<String> {
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
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

/// Test the test command in a specific directory using working_directory parameter
pub async fn test_test_command(project_path: &str) -> Result<String> {
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "test".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test adding a dependency using working_directory parameter
pub async fn test_add_dependency(
    project_path: &str,
    dep_name: &str,
    version: Option<&str>,
) -> Result<String> {
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    let args = if let Some(v) = version {
        object!({ "name": dep_name, "working_directory": project_path, "version": v })
    } else {
        object!({ "name": dep_name, "working_directory": project_path })
    };

    let result = client
        .call_tool(CallToolRequestParam {
            name: "add".into(),
            arguments: Some(args),
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test removing a dependency using working_directory parameter
pub async fn test_remove_dependency(project_path: &str, dep_name: &str) -> Result<String> {
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "remove".into(),
            arguments: Some(object!({ "name": dep_name, "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test the update command using working_directory parameter
pub async fn test_update_command(project_path: &str) -> Result<String> {
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "update".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test the doc command in a specific directory using working_directory parameter
pub async fn test_doc_command(project_path: &str) -> Result<String> {
    let original_dir = env::current_dir()?;

    let bin = server_binary(&original_dir);
    let client = ()
        .serve(TokioChildProcess::new(Command::new(bin).configure(
            |cmd| {
                cmd.current_dir(&original_dir);
            },
        ))?)
        .await?;

    let result = client
        .call_tool(CallToolRequestParam {
            name: "doc".into(),
            arguments: Some(object!({ "working_directory": project_path })),
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}

/// Test the doc command with content extraction in a specific directory
pub async fn test_doc_command_with_content(project_path: &str) -> Result<String> {
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
            name: "doc".into(),
            arguments: Some(object!({
                "working_directory": project_path
            })),
        })
        .await?;

    client.cancel().await?;

    Ok(format!("{result:?}"))
}
