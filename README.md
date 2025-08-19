# Async Cargo MCP

<img src="./assets/async-cargo-mcp-logo.png" height="65" align="right" alt="Async Cargo MCP Logo"/>

[![CI](https://github.com/paulirotta/async_cargo_mcp/actions/workflows/rust.yml/badge.svg)](https://github.com/paulirotta/async_cargo_mcp/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache2](https://img.shields.io/badge/License-Apache2.0-yellow.svg)](https://opensource.org/license/apache-2-0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

<a href="https://github.com/paulirotta/async_cargo_mcp/security/code-scanning">
    <img src="assets/codeql.svg" height="35" alt="CodeQL Static Analysis"/>
<a>

A Model Context Protocol (MCP) server that enables AI assistants to execute Rust cargo commands safely with asynchronous operation support.

## Overview

This MCP server allows AI to run cargo commands quickly in the background. Multiple commands can be queued and executed while the AI continues to think. This helps complete tasks faster and, with fewer interruptions, may use less AI premium tokens.

Most tool-calling AIs take advantage of this. All LLMs are equal, some are more equal than others. We continue to iterate and help is welcome.

### Example: `Claude Sonnet 4`

> **Now let me run the full test suite to verify that all tests pass with my fixes:**
>
> ---
>
> **async_cargo_mcp (MCP Server):**
>
> > **Perfect! The test is running in the background. While it's running, let me update the `agent-plan.md` to mark this task as substantially complete and document the final findings:**
>
> ```
> agent-plan.md+3-3
> ```

### Example: `GPT-5`

> **I'll run clippy fix, build, nextest, and fmt across the workspace to ensure the changes compile and pass checks.**
>
> ```
> async_cargo_mcp (MCP Server)
> async_cargo_mcp (MCP Server)
> async_cargo_mcp (MCP Server)
> async_cargo_mcp (MCP Server)
> async_cargo_mcp (MCP Server)
> ⬇️ agent-plan.md <span style="color:green;">+4</span> <span style="color:red;">-5</span>
> ```

### Example: `Gemini 2.5 Pro`

> Now, I'll run cargo build to check for any remaining issues.
>
> async_cargo_mcp (MCP Server)
> I've addressed the let chain and several polars import errors, including adding imports for col and lit. I am now rebuilding the project to check for any remaining issues.
>
> async_cargo_mcp (MCP Server)

After installing `async_cargo_mcp`, you save development time by enabling your LLM to launch background Rust Cargo operations while continuing to think or update planning documents. LLMs can choose synchronous or asynchronous execution. For long-running tasks, asynchronous operations let the LLM proceed with other work while this tool builds and tests in the background. Callbacks are part of the MCP specification. The implementation uses [Anthropic's official `rmcp` Rust SDK](https://github.com/modelcontextprotocol/rust-sdk).

### High-Performance Shell Pool Architecture

The server features a **pre-warmed shell pool system** that provides **10x faster command execution**. This reduces command startup latency from 50-200ms to just 5-20ms, delivering rapid responses during development workflows.

More in the [Shell Pool Guide](SHELL_POOL_GUIDE.md).

## Supported Commands

### Core Cargo Commands

- **`build`** - Compile the current package
- **`run`** - Build and execute the binary
- **`test`** - Run the test suite
- **`check`** - Check for compile errors without building
- **`clean`** - Remove build artifacts
- **`doc`** - Build documentation
- **`add`** - Add dependencies to Cargo.toml (updates `Cargo.toml` so synchronous)
- **`remove`** - Remove dependencies from Cargo.toml (synchronous)
- **`update`** - Update dependencies to latest compatible versions (synchronous)
- **`fetch`** - Download dependencies without building
- **`install`** - Install a Rust binary
- **`search`** - Search for packages on crates.io
- **`tree`** - Display dependency tree (synchronous)
- **`version`** - Show cargo version information (synchronous)
- **`rustc`** - Compile with custom rustc options
- **`metadata`** - Output package metadata as JSON (synchronous)

### Extension Commands (if installed)

- **`clippy`** - Enhanced linting and code quality checks
- **`nextest`** - Faster test execution
- **`fmt`** - Code formatting with rustfmt
- **`audit`** - Security vulnerability scanning
- **`upgrade`** - Upgrade dependencies to latest versions (synchronous)
- **`bench`** - Run benchmarks

### Control Commands

- **`wait`** - Wait for async operations to complete (synchronous)

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
      "cwd": "${workspaceFolder}",
      "command": "cargo",
      "args": ["run", "--release", "--bin", "async_cargo_mcp"]
    }
  },
  "inputs": []
}
```

Copy and edit to taste the optional [Rust_Beast_Mode.chatmode.md](./.github/chatmodes/Rust_Beast_Mode.chatmode.md) to help your LLM with tool use.

Restart VSCode to activate the server.

## Shell Pool Configuration

The server automatically manages pre-warmed shell pools for optimal performance. You can customize the behavior using command-line arguments:

```bash
# Configure shell pool size (default: 2 shells per directory)
cargo run --release -- --shell-pool-size 4

# Set maximum total shells across all pools (default: 20)
cargo run --release -- --max-shells 50

# Disable shell pools entirely (fallback to direct command spawning)
cargo run --release -- --disable-shell-pools

# Combine options as needed
cargo run --release -- --shell-pool-size 3 --max-shells 30
```

### Shell Pool Benefits

- **10x Performance**: Command startup reduced from 50-200ms to 5-20ms
- **Automatic Management**: Background health monitoring and cleanup
- **Transparent Operation**: Same API and behavior as before, just faster
- **Resource Efficient**: Idle shells are automatically cleaned up after 30 minutes

### Production Deployment

For production use, build with optimizations enabled:

```bash
cargo build --release
./target/release/async_cargo_mcp --shell-pool-size 3 --max-shells 25
```

## MCP tool usage instructions for AI

Commands support both synchronous and asynchronous execution. For long-running operations, enable async notifications:

```json
{
  "working_directory": "/path/to/project",
  "enable_async_notifications": true
}
```

When async is enabled, use the `wait` command to collect results:

- `wait` with `operation_ids` - wait for specific operations by providing their IDs (required)

## License

Licensed under either [Apache License 2.0](APACHE_LICENSE.txt) or [MIT License](MIT_LICENSE.txt).

## Acknowledgments

Built with [Anthropic's official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk).
