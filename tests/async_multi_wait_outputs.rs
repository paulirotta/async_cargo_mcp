//! Verify waiting for multiple async operations returns full outputs for each
//! Reproduces issue where FULL OUTPUT section is empty for some operations.

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
async fn test_async_multiple_builds_then_wait_returns_full_outputs() -> Result<()> {
    // Two separate temp projects so outputs differ
    let temp1 = create_basic_project().await?;
    let temp2 = create_basic_project().await?;
    let p1 = temp1.path().to_str().unwrap().to_string();
    let p2 = temp2.path().to_str().unwrap().to_string();

    // Start server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--").arg("--enable-wait");
            },
        ))?)
        .await?;

    // Kick off two async builds
    let r1 = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({"working_directory": p1, "enable_async_notification": true})),
        })
        .await?;
    let r2 = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({"working_directory": p2, "enable_async_notification": true})),
        })
        .await?;

    let t1 = format!("{:?}", r1.content);
    let t2 = format!("{:?}", r2.content);
    assert!(t1.contains("started at") && t2.contains("started at"));
    let id1 = extract_operation_id(&t1).expect("id1");
    let id2 = extract_operation_id(&t2).expect("id2");

    // Wait for both using operation_ids list
    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({"operation_ids": [id1.clone(), id2.clone()] })),
        })
        .await?;
    let wait_text = format!("{:?}", wait.content);

    // Should have two OPERATION COMPLETED sections with FULL OUTPUT each not immediately empty
    let completed_count = wait_text.matches("OPERATION COMPLETED").count();
    assert!(
        completed_count >= 2,
        "Expected at least two completed operations: {wait_text}"
    );
    // For each operation, ensure FULL OUTPUT section is not just header followed by \n\nOutput: (empty)
    for id in [&id1, &id2] {
        let marker = format!("OPERATION COMPLETED: '{id}'");
        assert!(
            wait_text.contains(&marker),
            "Missing marker for {id}: {wait_text}"
        );
    }
    assert!(
        wait_text.contains("=== FULL OUTPUT ==="),
        "Missing FULL OUTPUT marker"
    );

    // Heuristic: after FULL OUTPUT marker, we expect some cargo text like Compiling/Finished or "Build completed successfully"
    assert!(
        wait_text.contains("Compiling")
            || wait_text.contains("Finished")
            || wait_text.contains("Build completed successfully"),
        "Full output appears empty or truncated: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}
