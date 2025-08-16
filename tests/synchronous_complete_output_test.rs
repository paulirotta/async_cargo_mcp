//! Test to verify synchronous build returns complete output including all stderr/stdout
//! This test should initially fail if synchronous operations truncate output

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
async fn test_synchronous_build_returns_complete_output() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Run synchronous build (enable_async_notifications=false or not set)
    let result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": false
            })),
        })
        .await?;

    let text = format!("{:?}", result.content);

    // Should have an Output section
    assert!(text.contains("Output:"), "No Output section found: {text}");

    // Should contain complete cargo build information
    assert!(
        text.contains("Compiling") || text.contains("Finished"),
        "Expected complete cargo build output with 'Compiling' or 'Finished': {text}"
    );

    // Should be reasonably verbose (more than just a summary line)
    let output_section = if let Some(pos) = text.find("Output:") {
        &text[pos..]
    } else {
        &text
    };

    assert!(
        output_section.len() > 50,
        "Output section seems too short ({}), might be truncated: {output_section}",
        output_section.len()
    );

    // Should NOT be truncated - check for common truncation indicators
    assert!(
        !text.contains("...") && !text.contains("truncated") && !text.contains("(output limited)"),
        "Output appears to be truncated: {text}"
    );

    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_synchronous_check_returns_complete_output() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Run synchronous check
    let result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notifications": false
            })),
        })
        .await?;

    let text = format!("{:?}", result.content);

    // Should have an Output section
    assert!(text.contains("Output:"), "No Output section found: {text}");

    // Should contain complete cargo check information
    assert!(
        text.contains("Checking") || text.contains("Finished"),
        "Expected complete cargo check output with 'Checking' or 'Finished': {text}"
    );

    // Should be reasonably verbose
    let output_section = if let Some(pos) = text.find("Output:") {
        &text[pos..]
    } else {
        &text
    };

    assert!(
        output_section.len() > 30,
        "Output section seems too short ({}), might be truncated: {output_section}",
        output_section.len()
    );

    let _ = client.cancel().await;
    Ok(())
}
