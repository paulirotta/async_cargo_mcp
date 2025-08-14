//! TDD: Failing build should include stderr details ALSO in the Output section (merged output behavior)

use anyhow::Result; mod common; use common::test_project::create_basic_project; use rmcp::{ServiceExt, model::CallToolRequestParam, object, transport::{ConfigureCommandExt, TokioChildProcess}}; use tokio::process::Command; use std::fs;

#[tokio::test]
async fn test_build_failure_merges_stderr_into_output() -> Result<()> {
    let temp = create_basic_project().await?; let project_path = temp.path();
    // Break the code
    fs::write(project_path.join("src/main.rs"), "fn main(){ let x = ; }")?;

    let client = ().serve(TokioChildProcess::new(Command::new("cargo").configure(|cmd| { cmd.arg("run").arg("--bin").arg("async_cargo_mcp"); } ))?).await?;

    let result = client.call_tool(CallToolRequestParam { name: "build".into(), arguments: Some(object!({"working_directory": project_path.to_str().unwrap()})) }).await?;
    let text = format!("{:?}", result.content);
    assert!(text.contains("- Build failed"), "Expected build to fail: {text}");
    assert!(text.contains("Error:"), "Should contain Error section with stderr");
    // NEW requirement: Output section should not be empty; should echo some error token like 'expected'
    // This currently FAILS until merge implemented
    let after_output = text.split("Output:").nth(1).unwrap_or("").trim();
    assert!(after_output.contains("expected") || after_output.contains("error") , "Merged Output section missing stderr content: {text}");

    let _ = client.cancel().await; Ok(()) }
