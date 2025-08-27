//! Integration tests for .cargo-lock remediation and timeout guidance

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

async fn create_basic_project() -> Result<TempDir> {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo_mcp_lock_test_")
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

    // Create src directory and main.rs
    fs::create_dir_all(project_path.join("src")).await?;
    fs::write(
        project_path.join("src").join("main.rs"),
        r#"fn main() { println!("hi"); }"#,
    )
    .await?;

    Ok(temp_dir)
}

async fn ensure_lock_async(project_dir: &Path) -> PathBuf {
    let lock_path = project_dir.join("target").join(".cargo-lock");
    if let Some(parent) = lock_path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    let _ = fs::write(&lock_path, b"lock").await;
    lock_path
}

#[tokio::test]
async fn wait_timeout_with_lock_detects_and_guides() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_path_buf();
    let lock_path = ensure_lock_async(&project_path).await;

    // Start server with small timeout and wait tool enabled
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--timeout")
                    .arg("1");
            },
        ))?)
        .await?;

    // Start a long sleep "operation" to ensure wait hits timeout
    let start_sleep = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(
                object!({"duration_ms": 3000, "working_directory": project_path.to_str().unwrap()}),
            ),
        })
        .await?;
    let _ = format!("{:?}", start_sleep.content);

    // Wait for it with timeout
    let wait_res = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({"operation_ids": ["op_sleep_0"]})),
        })
        .await?;

    let text = format!("{:?}", wait_res.content);
    assert!(text.contains("Wait timed out"));
    assert!(text.contains(".cargo-lock"));
    assert!(text.contains(lock_path.to_str().unwrap()));
    assert!(text.contains("cargo_lock_remediation"));
    Ok(())
}

#[tokio::test]
async fn remediation_tool_deletes_lock_and_optionally_cleans() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_path_buf();
    let lock_path = ensure_lock_async(&project_path).await;

    // Start server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Call remediation action B (delete only)
    let res_b = client
        .call_tool(CallToolRequestParam {
            name: "cargo_lock_remediation".into(),
            arguments: Some(object!({
                "working_directory": project_path.to_str().unwrap(),
                "action": "B"
            })),
        })
        .await?;
    let text_b = format!("{:?}", res_b.content);
    assert!(text_b.contains("Result:"));
    assert!(
        !lock_path.exists(),
        ".cargo-lock should be deleted by action B"
    );

    // Recreate lock and call action C (no-op)
    let _ = ensure_lock_async(&project_path).await;
    let res_c = client
        .call_tool(CallToolRequestParam {
            name: "cargo_lock_remediation".into(),
            arguments: Some(object!({
                "working_directory": project_path.to_str().unwrap(),
                "action": "C"
            })),
        })
        .await?;
    let text_c = format!("{:?}", res_c.content);
    assert!(text_c.contains("No action taken"));
    assert!(project_path.join("target").join(".cargo-lock").exists());

    Ok(())
}
