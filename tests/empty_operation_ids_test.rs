//! Test that verifies empty operation_ids list is properly rejected

use anyhow::Result;
mod common;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

#[tokio::test]
async fn test_wait_with_empty_operation_ids_fails() -> Result<()> {
    // Start the MCP server with --enable-wait flag to test legacy behavior
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Try to wait with an empty operation_ids array
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": [] })),
        })
        .await;

    // This should fail with a clear error message
    match wait_result {
        Err(e) => {
            let error_str = format!("{}", e);
            assert!(
                error_str.contains("operation_ids cannot be empty")
                    || error_str.contains("at least one operation ID"),
                "Expected clear error about empty operation_ids, got: {error_str}"
            );
        }
        Ok(result) => {
            // If it doesn't error during the call, check if the result contains an error message
            let result_text = format!("{:?}", result);
            assert!(
                result_text.contains("cannot be empty")
                    || result_text.contains("at least one operation"),
                "Expected error about empty operation_ids, got success: {result_text}"
            );
        }
    }

    let _ = client.cancel().await;
    Ok(())
}
