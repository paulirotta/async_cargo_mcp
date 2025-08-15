//! Failing-first test: synchronous `clippy` should include stderr compile lines (and clippy diagnostics) in Output section.
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
async fn test_synchronous_clippy_includes_compile_stderr() -> Result<()> {
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
            name: "clippy".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "args": ["--fix", "--tests", "--allow-dirty", "--allow-no-vcs"],
                "enable_async_notifications": false
            })),
        })
        .await?;

    let text = format!("{:?}", result.content);
    assert!(text.contains("Output:"), "No Output section: {text}");
    // Require compile line from stderr to ensure stderr merging implemented.
    let has_compile_or_check = text.contains("Compiling") || text.contains("Checking");
    assert!(
        has_compile_or_check,
        "Expected 'Compiling' or 'Checking' line from stderr merged into Output but missing. Got: {text}"
    );
    // Ensure output is verbose enough (arbitrary minimal length > 120 chars after Output:)
    let verbose_len = text.len();
    assert!(
        verbose_len > 120,
        "Expected verbose clippy output length >120, got {verbose_len}: {text}"
    );
    let _ = client.cancel().await;
    Ok(())
}
