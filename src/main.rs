//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

mod cargo_command;

use anyhow::Result;
use clap::Parser;

/// Async Cargo MCP Server
///
/// A Model Context Protocol (MCP) server that provides asynchronous cargo command execution.
/// This server allows LLMs to interact with Rust cargo commands either synchronously or
/// asynchronously, enabling continued processing while waiting for command responses.
#[derive(Parser, Debug)]
#[command(
    name = "async_cargo_mcp",
    version = env!("CARGO_PKG_VERSION"),
    about = "Async Cargo MCP Server - Execute cargo commands concurrent with other LLM and MCP operations via Model Context Protocol",
    long_about = "A Model Context Protocol (MCP) server that provides asynchronous cargo command execution.\n\
                  This server allows LLMs to interact with Rust cargo commands returning results either synchronously or\n\
                  asynchronously, enabling continued processing while waiting for command responses.\n\n\
                  Available commands include: build, run, test, check, add, remove, update, and counter operations."
)]
struct Args {
    /// Enable asynchronous command execution (default: true)
    ///
    /// When true, commands are spawned asynchronously allowing the LLM to continue processing
    /// while waiting for results. When false, commands run synchronously and block until completion.
    /// Some LLMs may require synchronous mode for compatibility.
    #[arg(
        long,
        default_value = "true",
        help = "Enable asynchronous command execution"
    )]
    spawn: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let spawn = args.spawn;

    println!(
        "async_cargo_mcp v{version}",
        version = env!("CARGO_PKG_VERSION")
    );
    println!("Spawn mode: {spawn}");

    println!("async_cargo_mcp stopped");

    Ok(())
}
