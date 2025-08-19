//! Test to verify that synchronous operations don't return operation IDs that can be waited on
//! This prevents LLMs from trying to wait for synchronous operations

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

#[tokio::test]
async fn test_synchronous_operations_no_operation_id() -> Result<()> {
    let temp = create_basic_project().await?;
    let working_dir = temp.path().to_str().unwrap().to_string();

    // Start server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test synchronous fmt (enable_async_notification = false or not set)
    let fmt_result = client
        .call_tool(CallToolRequestParam {
            name: "fmt".into(),
            arguments: Some(object!({
                "working_directory": working_dir.clone(),
                "enable_async_notification": false  // Explicit synchronous
            })),
        })
        .await?;

    let fmt_text = format!("{:?}", fmt_result.content);

    // Should NOT contain operation ID for synchronous execution
    assert!(
        !fmt_text.contains("op_"),
        "Synchronous fmt should not return operation ID: {fmt_text}"
    );

    // Should contain completion message instead
    assert!(
        fmt_text.contains("completed")
            || fmt_text.contains("formatted")
            || fmt_text.contains("successful"),
        "Synchronous fmt should contain completion message: {fmt_text}"
    );

    // Test other synchronous commands
    let version_result = client
        .call_tool(CallToolRequestParam {
            name: "version".into(),
            arguments: Some(object!({})),
        })
        .await?;

    let version_text = format!("{:?}", version_result.content);

    // Version should never have operation ID as it's always synchronous
    assert!(
        !version_text.contains("op_"),
        "Version command should never return operation ID: {version_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_async_operations_do_return_operation_id() -> Result<()> {
    let temp = create_basic_project().await?;
    let working_dir = temp.path().to_str().unwrap().to_string();

    // Start server
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test asynchronous fmt (enable_async_notification = true)
    let fmt_result = client
        .call_tool(CallToolRequestParam {
            name: "fmt".into(),
            arguments: Some(object!({
                "working_directory": working_dir.clone(),
                "enable_async_notification": true  // Explicit asynchronous
            })),
        })
        .await?;

    let fmt_text = format!("{:?}", fmt_result.content);

    // Should contain operation ID for asynchronous execution
    assert!(
        fmt_text.contains("op_"),
        "Asynchronous fmt should return operation ID: {fmt_text}"
    );

    // Should contain "started in the background" or similar message
    assert!(
        fmt_text.contains("background") || fmt_text.contains("started"),
        "Asynchronous fmt should indicate background operation: {fmt_text}"
    );

    let _ = client.cancel().await;
    Ok(())
}
