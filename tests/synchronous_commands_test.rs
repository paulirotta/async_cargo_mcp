//! Test to verify that synchronous commands do not support async operations
//! This test ensures that commands marked as synchronous in README behave consistently.

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
async fn test_synchronous_commands_do_not_support_async() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test commands that should be synchronous according to README
    let synchronous_commands = vec![
        (
            "update",
            "Update dependencies to latest compatible versions (synchronous)",
        ),
        ("tree", "Display dependency tree (synchronous)"),
        ("version", "Show cargo version information (synchronous)"),
        ("metadata", "Output package metadata as JSON (synchronous)"),
    ];

    for (command, description) in synchronous_commands {
        println!("Testing synchronous command: {} - {}", command, description);

        let mut args = object!({
            "enable_async_notifications": true
        });

        // Add working_directory for commands that need it
        if command != "version" {
            args.insert(
                "working_directory".to_string(),
                serde_json::Value::String(project_path.clone()),
            );
        }

        let result = client
            .call_tool(CallToolRequestParam {
                name: command.into(),
                arguments: Some(args),
            })
            .await?;

        let response_text = format!("{:?}", result.content);
        println!("Response for {}: {}", command, response_text);

        // Synchronous commands should NOT return "started in background" messages
        assert!(
            !response_text.contains("started in background"),
            "Command '{}' should be synchronous but returned async response: {}",
            command,
            response_text
        );

        // Synchronous commands should return immediate completion
        assert!(
            response_text.contains("completed successfully")
                || response_text.contains("operation")
                || response_text.contains("failed"),
            "Command '{}' should return immediate result: {}",
            command,
            response_text
        );
    }

    client.cancel().await?;
    Ok(())
}
