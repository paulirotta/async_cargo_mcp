//! Verify async check returns full output via wait and progress notifications

use anyhow::Result;
use async_cargo_mcp::tool_hints;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::fs;
use tempfile::TempDir;
use tokio::process::Command;

#[tokio::test]
async fn test_async_check_then_wait_returns_full_output() -> Result<()> {
    // Create a minimal cargo project
    let temp = create_test_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    // Start the MCP server via cargo run
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Kick off async check
    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": true
            })),
        })
        .await?;

    let first_text = format!("{:?}", check_result.content);
    assert!(first_text.contains("started in background"));

    let op_id = extract_operation_id(&first_text).expect("operation id should be present");
    assert!(op_id.starts_with("op_"));

    // Verify the standardized preview() hint is included (accept raw or debug-escaped forms)
    let expected_hint = tool_hints::preview(&op_id, "check");
    let expected_hint_debug = expected_hint.replace('\n', "\\n");
    assert!(
        first_text.contains(&expected_hint_debug) || first_text.contains(&expected_hint),
        "Initial async response must include preview() content.\nExpected preview (raw or debug-escaped):\n{expected_hint}\n--- Escaped ---\n{expected_hint_debug}\nGot:\n{first_text}"
    );

    // Wait for completion
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_id": op_id, "timeout_secs": 180 })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);
    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));
    assert!(
        wait_text.contains("=== FULL"),
        "Wait output should contain full output marker: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

fn extract_operation_id(s: &str) -> Option<String> {
    if let Some(start) = s.find("op_") {
        let rest = &s[start..];
        let mut id = String::new();
        for ch in rest.chars() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                id.push(ch);
            } else {
                break;
            }
        }
        if id.starts_with("op_") {
            return Some(id);
        }
    }
    None
}

async fn create_test_project() -> Result<TempDir> {
    let dir = tempfile::Builder::new()
        .prefix("cargo_mcp_async_check_")
        .tempdir()?;
    let path = dir.path();

    fs::write(
        path.join("Cargo.toml"),
        r#"[package]
name = "test_project_check"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )?;

    fs::create_dir(path.join("src"))?;
    fs::write(
        path.join("src/main.rs"),
        r#"fn main() { println!("hello"); }
"#,
    )?;

    Ok(dir)
}
