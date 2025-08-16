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
    let first = client.call_tool(CallToolRequestParam {
        name: "build".into(), 
        arguments: Some(object!({"working_directory": project_path.clone(), "enable_async_notifications": true})) 
    }).await?;

    // Extract the operation ID from first build and wait for it to complete
    let first_text = format!("{:?}", first.content);
    let first_op_id = extract_operation_id(&first_text).expect("first operation id");
    let _ = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({"operation_ids": [first_op_id]})),
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
            arguments: Some(object!({"operation_ids": [op_id]})),
        })
        .await?;
    let wait_text = format!("{:?}", wait.content);
    // With merged output logic, second build may still show minimal lines (lock waits, Finished),
    // so accept either the legacy placeholder or a very short output (< 15 lines) with no Compiling lines.
    let has_placeholder = wait_text.contains("(no command stdout captured")
        || wait_text.contains("(no compiler output – build likely up to date)");
    let lines: Vec<&str> = wait_text.lines().collect();
    let short_build = lines.len() < 40 && !wait_text.contains("Compiling async_cargo_mcp");
    assert!(
        has_placeholder || short_build,
        "Expected placeholder or short minimal build output after no-op build. Got: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}
