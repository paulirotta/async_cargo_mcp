//! Test to verify async operations return complete output in WAIT results
//! This test should demonstrate if async operations truncate output differently than sync

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
async fn test_async_build_wait_returns_complete_output_detailed() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Run async build
    let result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let initial_text = format!("{:?}", result.content);
    let op_id = extract_operation_id(&initial_text).expect("Should have operation ID");

    // Wait for completion
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [op_id] })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Print the actual output length for debugging
    println!("Wait result length: {}", wait_text.len());
    println!(
        "Wait result (first 500 chars): {}",
        &wait_text[..wait_text.len().min(500)]
    );

    // Should contain FULL OUTPUT marker
    assert!(
        wait_text.contains("=== FULL OUTPUT ==="),
        "Wait result should contain FULL OUTPUT marker: {wait_text}"
    );

    // Should contain detailed cargo output
    assert!(
        wait_text.contains("Compiling") || wait_text.contains("Finished"),
        "Wait result should contain detailed cargo output: {wait_text}"
    );

    // Should be reasonably long (not truncated)
    assert!(
        wait_text.len() > 200,
        "Wait result seems too short ({}), might be truncated: first 200 chars: {}",
        wait_text.len(),
        &wait_text[..wait_text.len().min(200)]
    );

    // Should contain the working directory in output
    assert!(
        wait_text.contains("Working Directory:"),
        "Wait result should include working directory information: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_multiple_async_operations_complete_output() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Launch multiple async operations
    let build_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;
    let build_text = format!("{:?}", build_result.content);
    let build_id = extract_operation_id(&build_text).expect("Should have build operation ID");

    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;
    let check_text = format!("{:?}", check_result.content);
    let check_id = extract_operation_id(&check_text).expect("Should have check operation ID");

    // Wait for both operations
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": [build_id.clone(), check_id.clone()]
            })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    println!("Multi-operation wait result length: {}", wait_text.len());

    // Should contain results for both operations
    assert!(
        wait_text.contains(&build_id),
        "Wait result should contain build operation ID {}: {wait_text}",
        build_id
    );
    assert!(
        wait_text.contains(&check_id),
        "Wait result should contain check operation ID {}: {wait_text}",
        check_id
    );

    // Should contain at least two OPERATION COMPLETED sections
    let completed_count = wait_text.matches("OPERATION COMPLETED").count();
    assert!(
        completed_count >= 2,
        "Expected at least 2 OPERATION COMPLETED sections, found {}: {wait_text}",
        completed_count
    );

    // Should be reasonably long for two complete operations
    assert!(
        wait_text.len() > 400,
        "Multi-operation wait result seems too short ({}), might be truncated",
        wait_text.len()
    );

    let _ = client.cancel().await;
    Ok(())
}
