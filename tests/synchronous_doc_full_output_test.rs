//! Failing-first test: synchronous `doc` should include stderr compile lines in Output section.

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
    let text = format!("{:?}", result.content);
    assert!(
        text.contains("Documentation generation completed successfully"),
        "Doc success message missing: {text}"
    );
    assert!(text.contains("Output:"), "No Output section: {text}");
    let has_compile_or_documenting =
        text.contains("Compiling test_project") || text.contains("Documenting test_project");
    assert!(
        has_compile_or_documenting,
        "Expected compile/documenting line merged into Output for doc command. Got: {text}"
    );
    let _ = client.cancel().await;
    Ok(())
}
