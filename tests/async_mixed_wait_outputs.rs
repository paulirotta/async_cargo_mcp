//! Test waiting on multiple operations where one succeeds and one fails, ensuring both full outputs are returned.

use anyhow::Result;
mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::fs;
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
async fn test_async_mixed_success_failure_wait_outputs() -> Result<()> {
    // Successful project
    let ok_proj = create_basic_project().await?;
    let ok_path = ok_proj.path().to_str().unwrap().to_string();
    // Failing project (introduce syntax error)
    let fail_proj = create_basic_project().await?;
    let fail_path = fail_proj.path().to_str().unwrap().to_string();
    let main_rs = format!("{}/src/main.rs", fail_path);
    let broken = "fn main() { let x = ; }";
    fs::write(&main_rs, broken)?;

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--").arg("--enable-wait");
            },
        ))?)
        .await?;

    let ok_resp = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(
                object!({"working_directory": ok_path, "enable_async_notification": true}),
            ),
        })
        .await?;
    let fail_resp = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(
                object!({"working_directory": fail_path, "enable_async_notification": true}),
            ),
        })
        .await?;

    let ok_text = format!("{:?}", ok_resp.content);
    let fail_text = format!("{:?}", fail_resp.content);
    let ok_id = extract_operation_id(&ok_text).unwrap();
    let fail_id = extract_operation_id(&fail_text).unwrap();

    let wait = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({"operation_ids": [ok_id.clone(), fail_id.clone()] })),
        })
        .await?;
    let wait_text = format!("{:?}", wait.content);

    assert!(wait_text.contains("OPERATION COMPLETED") || wait_text.contains("OPERATION FAILED"));
    // Both ids present
    assert!(wait_text.contains(&ok_id));
    assert!(wait_text.contains(&fail_id));
    // Failure should include FULL ERROR OUTPUT marker
    assert!(
        wait_text.contains("=== FULL ERROR OUTPUT ===") || wait_text.contains("- Build failed"),
        "Expected error output marker: {wait_text}"
    );
    // Success should include success phrase
    assert!(
        wait_text.contains("Build completed successfully"),
        "Expected success output: {wait_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}
