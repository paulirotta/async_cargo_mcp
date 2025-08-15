//! Failing-first tests: ensure additional synchronous commands merge stderr into Output or supply placeholder.
//! Targets: fix, bench, clean (representative of missing merge implementations per inventory).

use anyhow::Result;
mod common;
use common::test_project::{create_basic_project, create_project_with_warning};
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

fn extract(raw: &str) -> String {
    raw.to_string()
}

#[tokio::test]
async fn test_synchronous_fix_merges_stderr_compile_lines() -> Result<()> {
    // Use project with warning to ensure fix triggers compilation on stderr.
    let temp = create_project_with_warning().await?;
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
            name: "fix".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": false}),
            ),
        })
        .await?;
    let text = format!("{:?}", result.content);
    let output = extract(&text);
    assert!(
        output.contains("Output:"),
        "Expected Output section: {output}"
    );
    // Expect compile line from stderr merged into Output once implementation added
    assert!(
        output.contains("Compiling") || output.contains("Checking"),
        "Expected compile/check stderr line merged into Output for fix command. Got: {output}"
    );
    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_synchronous_bench_merges_stderr_compile_lines() -> Result<()> {
    let temp = create_basic_project().await?; // single bench may not exist but cargo will still compile
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
            name: "bench".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": false}),
            ),
        })
        .await?;
    let text = format!("{:?}", result.content);
    assert!(text.contains("Output:"), "Expected Output section: {text}");
    assert!(
        text.contains("Compiling") || text.contains("Checking"),
        "Expected compile/check stderr line merged into Output for bench command. Got: {text}"
    );
    let _ = client.cancel().await;
    Ok(())
}

#[tokio::test]
async fn test_synchronous_clean_has_placeholder_when_no_output() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap();
    // Run build first to create target so clean may have minimal output
    {
        let client = ()
            .serve(TokioChildProcess::new(Command::new("cargo").configure(
                |cmd| {
                    cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
                },
            ))?)
            .await?;
        let _ = client
            .call_tool(CallToolRequestParam { name: "build".into(), arguments: Some(object!({"working_directory": project_path, "enable_async_notifications": false})) })
            .await?;
        let _ = client.cancel().await;
    }
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;
    let result = client
        .call_tool(CallToolRequestParam {
            name: "clean".into(),
            arguments: Some(
                object!({"working_directory": project_path, "enable_async_notifications": false}),
            ),
        })
        .await?;
    let text = format!("{:?}", result.content);
    assert!(
        text.contains("Output:"),
        "Expected Output section for clean: {text}"
    );
    // Accept either placeholder (empty output case) OR any non-empty output content
    if !(text.contains("(no clean output captured)")) {
        // Verify there is some non-whitespace content after "Output:" label
        if let Some(pos) = text.find("Output:") {
            let after = &text[pos + "Output:".len()..];
            assert!(
                !after.trim().is_empty(),
                "Expected placeholder or some output after Output:. Got: {text}"
            );
        } else {
            panic!("Missing Output: label in clean command output: {text}");
        }
    }
    let _ = client.cancel().await;
    Ok(())
}
