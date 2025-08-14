//! Test that operations with no stdout still return a meaningful placeholder in FULL OUTPUT via wait

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
async fn test_wait_returns_placeholder_for_empty_output() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // First build to compile (produces output); second build should be mostly silent
    let _first = client.call_tool(CallToolRequestParam { name: "build".into(), arguments: Some(object!({"working_directory": project_path.clone(), "enable_async_notifications": true})) }).await?;
    // Wait for first to finish to ensure subsequent build has no work
    // Use wait for all
    let _ = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: None,
        })
        .await?;

    // Start second build (expected to have empty stdout)
    let second = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": true}),
            ),
        })
        .await?;
    let text = format!("{:?}", second.content);
    assert!(text.contains("started in background"));
    let op_id = extract_operation_id(&text).expect("operation id");

    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({"operation_id": op_id})),
        })
        .await?;
    let wait_text = format!("{:?}", wait.content);
    assert!(
        wait_text.contains("(no command stdout captured"),
        "Expected placeholder in wait output: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}
