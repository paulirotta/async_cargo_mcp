//! # Async Cargo MCP
//!
//! A Model Context Protocol (MCP) server that provides asynchronous access to Cargo commands.
//! This crate allows clients (such as language models and development tools) to execute Rust
//! build system commands like `build`, `test`, `run`, `doc`, and dependency management operations
//! through a standardized protocol interface.
//!
//! ## Key Features
//!
//! - **Asynchronous Operations**: All Cargo commands execute asynchronously with optional progress callbacks
//! - **MCP Protocol Compliance**: Full implementation of the Model Context Protocol for tool integration
//! - **Documentation Generation**: Generate and access comprehensive API documentation via the `doc` command
//! - **Working Directory Support**: Execute commands in any specified directory
//! - **Comprehensive Cargo Support**: Build, test, run, check, add/remove dependencies, update, and documentation generation
//! - **Progress Monitoring**: Real-time feedback and operation monitoring for long-running tasks
//!
//! ## Available Commands
//!
//! ### Core Cargo Operations
//! - `build`: Compile the project using `cargo build`
//! - `test`: Run tests using `cargo test`  
//! - `run`: Execute the project using `cargo run`
//! - `check`: Check for errors without building using `cargo check`
//! - `doc`: Generate documentation using `cargo doc --no-deps`
//!
//! ### Dependency Management
//! - `add`: Add dependencies using `cargo add`
//! - `remove`: Remove dependencies using `cargo remove`
//! - `update`: Update dependencies using `cargo update`
//!
//! ## Documentation Generation and Usage
//!
//! The `doc` command is particularly valuable for LLMs and development tools as it generates
//! comprehensive API documentation that can be accessed at `target/doc/[crate_name]/index.html`.
//! This documentation provides up-to-date API information that complements source code analysis,
//! similar to popular documentation tools but specifically tailored for the current project state.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use async_cargo_mcp::test_doc_functionality;
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Test documentation generation
//!     let results = test_doc_functionality().await?;
//!     println!("Test results: {}", results);
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`cargo_tools`]: Core implementation of Cargo command handlers and MCP tool interface
//! - [`callback_system`]: Asynchronous callback management for progress updates and notifications  
//! - [`command_registry`]: Command registration and dispatch system for MCP tools
//! - [`operation_monitor`]: Monitoring and lifecycle management for long-running operations
//! - [`test_cargo_tools`]: Integration testing utilities for Cargo commands with working directory support
//!
//! ## Integration with Development Tools
//!
//! This crate is designed to be integrated into development environments, IDEs, and AI-powered
//! coding assistants that need programmatic access to Rust build tools. The MCP protocol ensures
//! standardized communication, while the async design allows for responsive user interfaces during
//! long-running operations.

pub mod callback_system;
pub mod cargo_tools;
pub mod command_registry;
pub mod mcp_callback;
pub mod operation_monitor;
pub mod test_cargo_tools;

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

/// Test documentation generation functionality
///
/// This function tests the `doc` command which generates comprehensive API documentation
/// for the current project using `cargo doc --no-deps`. This is particularly valuable for:
/// - Providing LLMs with up-to-date API information
/// - Ensuring documentation generation works correctly
/// - Verifying the doc command integration with the MCP server
///
/// The generated documentation serves as a complement to source code analysis,
/// similar to popular documentation tools but tailored for real-time project understanding.
///
/// # Returns
///
/// A `Result<String>` containing either:
/// - `Ok(String)`: Success message with path to generated documentation
/// - `Err(anyhow::Error)`: Error details if documentation generation fails
///
/// # Example
///
/// ```rust,no_run
/// use async_cargo_mcp::test_doc_functionality;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let doc_result = test_doc_functionality().await?;
///     println!("Documentation generation result:\n{}", doc_result);
///     Ok(())
/// }
/// ```
pub async fn test_doc_functionality() -> Result<String> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test doc command
    let doc_result = client
        .call_tool(CallToolRequestParam {
            name: "doc".into(),
            arguments: Some(object!({
                "working_directory": std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Doc command failed: {}", e))?;

    // Store the result before canceling the client
    let result = format!(
        "Documentation generation test results:\n- Doc result: {doc_result:?}"
    );

    // Cancel the client - ignore errors since transport might already be closed
    let _ = client.cancel().await;

    Ok(result)
}
