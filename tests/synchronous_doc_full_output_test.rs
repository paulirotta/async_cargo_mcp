//! Failing-first test: synchronous `doc` should include stderr compile lines in Output section.

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

#[tokio::test]
async fn test_synchronous_doc_includes_compile_stderr() -> Result<()> {
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
            name: "doc".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": false}),
            ),
        })
        .await?;
    let raw = format!("{:?}", result.content);
    let text = strip_ansi(&raw);
    assert!(
        text.contains("Documentation generation completed successfully"),
        "Doc success message missing: {text}"
    );
    assert!(text.contains("Output:"), "No Output section: {text}");
    // Accept presence of generic action keywords irrespective of project name to avoid brittleness
    let has_compile_or_documenting = text.contains("Compiling") || text.contains("Documenting");
    assert!(
        has_compile_or_documenting,
        "Expected compile/documenting line merged into Output for doc command. Got: {text}"
    );
    let _ = client.cancel().await;
    Ok(())
}
