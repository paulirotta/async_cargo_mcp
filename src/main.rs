//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

mod cargo_command;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("async_cargo_mcp is running");

    println!("async_cargo_mcp stopped");

    Ok(())
}
