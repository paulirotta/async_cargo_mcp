//! Failing-first tests to ensure synchronous commands return FULL stdout+stderr
//! specifically that stderr compilation lines are present in the reported Output.

use anyhow::Result;
mod common;
use crate::common::strip_ansi;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

fn extract_tool_output(raw: &str) -> String {
    raw.to_string()
}

#[tokio::test]
async fn test_synchronous_run_includes_compile_stderr() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;
    let result = client
        .call_tool(CallToolRequestParam {
            name: "run".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": false}),
            ),
        })
        .await?;
    let raw = format!("{:?}", result.content);
    let output = strip_ansi(&extract_tool_output(&raw));
    assert!(output.contains("Output:"), "No Output section: {output}");
    assert!(
        output.contains("Compiling") || output.contains("Checking"),
        "Expected stderr compile/check line merged into Output but missing. Got: {output}"
    );
    assert!(
        output.contains("Finished"),
        "Expected Finished line from stderr. Got: {output}"
    );
    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_synchronous_test_includes_compile_stderr() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;
    let result = client
        .call_tool(CallToolRequestParam {
            name: "test".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": false}),
            ),
        })
        .await?;
    let raw = format!("{:?}", result.content);
    let output = strip_ansi(&extract_tool_output(&raw));
    assert!(output.contains("Output:"), "No Output section: {output}");
    assert!(
        output.contains("Compiling") || output.contains("Checking"),
        "Expected compile/check stderr line inside Output for test command. Got: {output}"
    );
    assert!(
        output.contains("test result"),
        "Expected test summary line. Got: {output}"
    );
    let _ = client.cancel().await;
    Ok(())
}
