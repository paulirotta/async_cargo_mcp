# Async Cargo MCP

<img src="./assets/async-cargo-mcp-logo.png" height="65" align="right" alt="Async Cargo MCP Logo"/>

[![CI](https://github.com/paulirotta/async_cargo_mcp/actions/workflows/rust.yml/badge.svg)](https://github.com/paulirotta/async_cargo_mcp/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

A Model Context Protocol (MCP) server that enables AI assistants to execute Rust cargo commands safely with asynchronous operation support.

## Overview

This MCP server allows AI assistants to run cargo commands in the background while continuing other tasks. Operations can be executed synchronously or asynchronously, with real-time progress notifications and comprehensive error handling.

## Asynchronous Workflow

This tool enables AI assistants to run cargo commands in the background while continuing other tasks. When async notifications are enabled, operations return immediately with an operation ID, allowing the AI to proceed with other work.

Example workflow:
1. AI starts a test suite with `enable_async_notifications: true`
2. Operation returns immediately with operation ID
3. AI continues with documentation or code updates
4. AI calls `wait` when ready to review test results

Example job start from the point of view of the LLM running this MCP service:
> **Input**
> ```json
> {
>   "working_directory": "/Users/paul/github/nb_lifeline3/rust/ml_tools",
>   "args": [
>     "--fix",
>     "--allow-dirty"
>   ],
>   "enable_async_notifications": true
> }
> ```
> **Output**
> ```
> Clippy operation op_43831394 started in background.
> 
> *** CRITICAL Tool Hint for LLMs ***: Operation op_43831394 is running in the background.
> *** DO NOT PROCEED assuming the operation is complete based on this message alone! ***
> *** You must wait for completion to get actual results (success/failure/output)! ***
> Next step: When you are ready to consume results, call `mcp_async_cargo_m_wait` with operation_id='op_43831394'.
> To wait for all pending operations, call `mcp_async_cargo_m_wait` with no operation_id.
> 
> **Always use async_cargo_mcp MCP tools** for ALL cargo operations. **Do not run cargo in a terminal.**
> While tools run, keep planning/writing code/tests. When you’re ready to use results, wait for the specific operation(s) you need next.
> You will receive a notifications with results when the clippy linting finishes.
> ```

Example Gemini 2.5 Pro:
> Now, I'll run cargo build to check for any remaining issues.
> 
> async_cargo_mcp (MCP Server)
> I've addressed the let chain and several polars import errors, including adding imports for col and lit. I am now rebuilding the project to check for any remaining issues.
> 
> async_cargo_mcp (MCP Server)
> 
> After installing `async_cargo_mcp`, you save development time by enabling your LLM to launch background Rust Cargo operations while continuing to think or update planning documents. LLMs can choose synchronous or asynchronous execution. For long-running tasks, asynchronous operations let the LLM proceed with other work while this tool builds and tests in the background. Callbacks are part of the MCP specification. The implementation uses [Anthropic's official `rmcp` Rust SDK](https://github.com/modelcontextprotocol/rust-sdk).


## Supported Commands

### Core Cargo Commands
- **`build`** - Compile the current package
- **`run`** - Build and execute the binary  
- **`test`** - Run the test suite
- **`check`** - Check for compile errors without building
- **`clean`** - Remove build artifacts
- **`doc`** - Build documentation
- **`add`** - Add dependencies to Cargo.toml
- **`remove`** - Remove dependencies from Cargo.toml
- **`update`** - Update dependencies to latest compatible versions
- **`fetch`** - Download dependencies without building
- **`install`** - Install a Rust binary
- **`search`** - Search for packages on crates.io
- **`tree`** - Display dependency tree
- **`version`** - Show cargo version information
- **`rustc`** - Compile with custom rustc options
- **`metadata`** - Output package metadata as JSON

### Extension Commands (if installed)
- **`clippy`** - Enhanced linting and code quality checks
- **`nextest`** - Faster test execution
- **`fmt`** - Code formatting with rustfmt
- **`audit`** - Security vulnerability scanning
- **`upgrade`** - Upgrade dependencies to latest versions
- **`bench`** - Run benchmarks

### Control Commands
- **`wait`** - Wait for async operations to complete

## Features

- **Asynchronous execution** with real-time progress updates
- **Safe operations** with proper working directory isolation
- **Type-safe parameters** with JSON schema validation
- **Operation monitoring** with timeout and cancellation support
- **Comprehensive error handling** and detailed logging

## Installation

```bash
git clone https://github.com/paulirotta/async_cargo_mcp.git
cd async_cargo_mcp
cargo build --release
```

## IDE Integration

### VSCode with GitHub Copilot

Enable MCP in VSCode settings:
```json
{
    "chat.mcp.enabled": true
}
```

Add the server configuration using `Ctrl/Cmd+Shift+P` → "MCP: Add Server":

```json
{
    "servers": {
        "async_cargo_mcp": {
            "type": "stdio",
            "command": "/path/to/async_cargo_mcp/target/release/async_cargo_mcp",
            "args": []
        }
    },
    "inputs": []
}
```

Restart VSCode to activate the server.

## MCP tool usage instructions for AI

Commands support both synchronous and asynchronous execution. For long-running operations, enable async notifications:

```json
{
    "working_directory": "/path/to/project",
    "enable_async_notifications": true
}
```

When async is enabled, use the `wait` command to collect results:
- `wait` with no parameters - wait for all operations
- `wait` with `operation_id` - wait for specific operation
- `wait` with `operation_ids` - wait for multiple operations

## License

Licensed under either [Apache License 2.0](APACHE_LICENSE.txt) or [MIT License](MIT_LICENSE.txt).

## Acknowledgments

Built with [Anthropic's official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk).
