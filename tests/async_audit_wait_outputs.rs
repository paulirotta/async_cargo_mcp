//! TDD: Ensure audit async wait returns combined stdout/stderr (stderr merged when stdout empty)

use anyhow::Result;
mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

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

#[tokio::test]
async fn test_async_audit_wait_combines_outputs() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    let resp = client
        .call_tool(CallToolRequestParam {
            name: "audit".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": true}),
            ),
        })
        .await?;
    let first = format!("{:?}", resp.content);
    assert!(
        first.contains("started in background")
            || first.contains("failed: cargo-audit is not installed")
    );
    if first.contains("failed: cargo-audit is not installed") {
        let _ = client.cancel().await;
        return Ok(());
    } // Skip if unavailable
    let op_id = extract_operation_id(&first).unwrap();

    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({"operation_id": op_id, "timeout_secs": 300})),
        })
        .await?;
    let wait_text = format!("{:?}", wait.content);
    assert!(wait_text.contains("=== FULL"));
    assert!(
        !wait_text.contains("Errors:"),
        "Expected merged output without separate Errors section: {wait_text}"
    );
    let _ = client.cancel().await;
    Ok(())
}
