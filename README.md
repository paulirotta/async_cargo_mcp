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

See the full product specification and roadmap: [product-specification.md](product-specification.md)

Most tool-calling AIs take advantage of this. All LLMs are equal, some are more equal than others. We continue to iterate and help is welcome.

### Example: `Claude Sonnet 4`

> **Now let me run the full test suite to verify that all tests pass with my fixes:**
>
> ```
> async_cargo_mcp (MCP Server):
>
> ```
>
> **Perfect! The test is running in the background. While it's running, let me update the `agent-plan.md` to mark this task as substantially complete and document the final findings:**
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
>
> ⬇️ agent-plan.md +4 -5
> ```

### Example: `Gemini 2.5 Pro`

> **Now, I'll rerun the tests to verify the fixes and the new title/label functionality.**
>
> ```
> async_cargo_mcp (MCP Server)
> ```
>
> **I have finished the first two tasks. I'll wait for the tests to finish, and if they pass, I'll start on removing...**
>
> **Then, I'll run the analysis scripts to verify everything.**
>
> ```
> async_cargo_mcp (MCP Server)
>
> Summarized conversation history
> ```
>
> **Whelp, the tests failed with a cascade of errors. It looks like I correctly updated...**

As you can see **(1) the developer**, **(2) the AI**, and **(3) Rust tooling** can be easily coordinated to all work productively and concurrently without loosing the storyline.

### High-Performance Shell Pool Architecture

The server features a **pre-warmed shell pool** that provides **10x faster command start** vs unpooled. Command startup latency from 50-200ms to just 5-20ms (Macbook Pro M1), delivering rapid responses while allowing stacking of commands. For example, `test` and `nextest` (after initial compile) do not hold the cargo filesystem lock while they run, allowing both the AI and other spawned cargo commands such as `clippy`to do useful work while they complete.

## Supported Commands

More information is available in `--help`.

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

- **`status`** - Query running operations status (non-blocking, returns JSON)
- **`wait`** - Wait for async operations to complete (synchronous, deprecated - results pushed automatically)
- **`cargo_lock_remediation`** - Safely handle `target/.cargo-lock` with options to delete and optionally `cargo clean` (synchronous, used as fallback when elicitation isn't available)

## Features

- **Asynchronous execution** with real-time progress updates
- **Automatic result push** - Operation results pushed to AI when complete (no manual wait required)
- **Safe operations** with proper working directory isolation
- **Type-safe parameters** with JSON schema validation
- **Operation monitoring** with timeout and cancellation support
- **Comprehensive error handling** and detailed logging
- **Concurrency metrics** for optimizing AI task parallelism
- **Status queries** - Non-blocking visibility into running operations

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

1. Edit the Json to taste. Also see the optional [Rust_Beast_Mode.chatmode.md](./.github/chatmodes/Rust_Beast_Mode.chatmode.md) including instructions to help your AI with tool use.

2. Restart VSCode to activate the server.

#### Synchronous Mode

These are good reasons to use `--synchronous` to have cargo commands run blocking the AI and terminal:

- you prefer less chatter of `waiting` in your AI dialogue
- you prefer to see the `cargo` command execute in your terminal
- you accept that the terminal is blocked while `cargo` commands execute
- your AI does not actually think or act on the next steps while waiting for cargo operations to complete
- you prefer to keep it simple and take your time to stop and drink coffee

```json
"args": ["run", "--release", "--bin", "async_cargo_mcp", "--", "--synchronous"]
```

Other command line arguments are less common. See `--help`.

## Shell Pool Configuration

The server automatically manages pre-warmed shell pools for optimal performance. You can customize the behavior using command-line arguments:

```bash
# Configure shell pool size (default: 2 shells per directory)
cargo run --release -- --shell-pool-size 4

# Set maximum total shells across all pools (default: 20)
cargo run --release -- --max-shells 50

# Disable shell pools entirely (fallback to direct command spawning)
cargo run --release -- --disable-shell-pools

# Disable specific tools (comma-separated list or repeat flag)
cargo run --release -- --disable add,remove,update,upgrade

# Force synchronous execution mode (disables async callbacks for all operations)
cargo run --release -- --synchronous

# Combine options as needed
cargo run --release -- --shell-pool-size 3 --max-shells 30 --synchronous
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
  "enable_async_notification": true
}
```

When async is enabled, prefer `status` to check progress. Use `wait` only if blocked and you need results to proceed:

- `wait` with `operation_ids` waits for specific operations by ID.

Notes about `wait` semantics:

- Available only in async mode (default). It’s not offered in synchronous mode.
- Only `operation_ids` are accepted; unknown fields are rejected. Configure timeouts via the server CLI (e.g., `--timeout 30`).
- On timeout, you’ll get a clear message including how long it waited. If a `target/.cargo-lock` issue is detected, the server suggests remediation using the `cargo_lock_remediation` tool.

### Execution Modes

- **Async Mode (default)**: Operations can run in the background with notifications when `enable_async_notification: true`. `wait` is available but discouraged; prefer `status`.
- **Synchronous Mode**: Use `--synchronous` to run all operations synchronously. `wait` is not offered in this mode.

### Selectively Disabling Tools

Operators can hide or block specific tools from being used with the `--disable <tool>` flag. You can pass a comma-separated list (preferred for multiple) or repeat the flag. This is useful for:

- Hardening production environments (e.g. disable `upgrade`, `audit`, or mutation-causing commands)
- Restricting heavy operations (`bench`, `nextest`) in resource-constrained contexts
- Enforcing a narrow AI action surface during experimentation

Examples:

```bash
cargo run --release -- --disable build,test,clippy

# Equivalent using repeated flags
cargo run --release -- --disable build --disable test --disable clippy
```

If a disabled tool is invoked by a client that cached an older schema, the server returns an error with marker `tool_disabled`.

## License

Licensed under either [Apache License 2.0](APACHE_LICENSE.txt) or [MIT License](MIT_LICENSE.txt).

## Acknowledgments

Built with [Anthropic's official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk).
