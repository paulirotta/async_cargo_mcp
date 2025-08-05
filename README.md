# Async Cargo MCP

<img src="assets/async-cargo-mcp-logo.png" height="65" align="right" alt="Async Cargo MCP Logo"/>

<a href="https://github.com/paulirotta/async_cargo_mcp/security/code-scanning">
    <img src="assets/codeql.svg" height="35" alt="CodeQL Static Analysis"/>
<a>

[![CI](https://github.com/paulirotta/async_cargo_mcp/actions/workflows/rust.yml/badge.svg)](https://github.com/paulirotta/async_cargo_mcp/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

**Model Context Protocol (MCP) server for Cargo with asynchronous response handling and comprehensive operation monitoring.**

This project provides a high-performance MCP server that allows Large Language Models (LLMs) to interact with Rust's Cargo build system operations. It supports both synchronous and asynchronous execution. For long-running tasks, asynchronous operations allow the LLM to continue with other tasks while the `async_cargo_mcp` process works in the background as a concurrent agent.

## Features

- **Walk and chew gum at the same time**: Long-running cargo commands return immediately. Callbacks notifify of progress. This allows the LLM to continue concurrent thinking or other tool commands. The LLM may not think of or expect this unless you prompt it.
- **Comprehensive Cargo Commands**: Implementation of all core cargo commands useful to an LLM: `build`, `test`, `run`, `check`, `doc`, `add`, `remove`, `update`, `clean`, `fix`, `search`, `bench`, `install`, `tree`, `version`, `fetch`, `rustc`, `metadata`
- **Optional Cargo Extension Commands**: If installed, the LLM can use:
    - `clippy` for enhanced linting and code quality checks
    - `nextest` for faster test execution
    - `upgrade` (from cargo-edit) for intelligent dependency upgrades and package management
    - `audit` (from cargo-audit) to audit Cargo.lock for crates with security vulnerabilities
    - `fmt` (from rustfmt) for code formatting
- **Typed Parameters**: Command parameters are strongly-typed with JSON schema validation to the the LLM on the straight-and-narrow path to success

## Status

It works with STDIN/STDOUT (not yet SSE), it is fast. It is not heavily field tested.
Here is an example of Claude Sonnet 4 using `async-cargo-mcp` and thinking at the same time:

> ..
> **Now let me run the full test suite to verify that all tests pass with my fixes:**
>
> ---
>
> **async_cargo_mcp (MCP Server):**
> > **Perfect! The test is running in the background. While it's running, let me update the `agent-plan.md` to mark this task as substantially complete and document the final findings:**
>
> ```
> agent-plan.md+3-3
> ```
> ..

### Current Capabilities
- All cargo commands implemented with fairly comprehensive integration test coverage
- MCP protocol official library integration with JSON schema validation tested in VSCode
- Async callback notifications for progress tracking, but LLMs may still ignore this and wait unless prompted
- `working_directory` is passed to commands, but we do not yet limit scope to one or more directory trees for safety

### Upcoming Features (open an issue with requests, PRs welcome)
- Better docs and examples
- Instructions for other IDEs and command line tools
- ´cargo install´ for easier setup
- VSCode plugin etc for easier setup
- Add RAG documentation server for the LLM to read current and upstream docs for the latest in-use API support (help the LLM with updates since its training cutoff date, similar to [Context7](https://context7.com/) but a different approach)
- [`cargo watch`](https://crates.io/crates/cargo-watch) integration for LLMs. Monitor and pre-emptively build etc so future commands return faster
- SSE support for other MCP setups (subject to security considerations)

## Installation

This project is under active development and can be installed from source:

```bash
git clone git@github.com:paulirotta/async_cargo_mcp.git
cd async_cargo_mcp
cargo build --release
```

## IDE Integration

### VSCode with GitHub Copilot

Ensure you have the internal MCP server enabled in your VSCode settings:

```json
"chat.mcp.enabled": true
```

Then, add the server to your VSCode configuration using `CTRL/CMD SHIFT P` and searching for "MCP: Add Server"

The server configuration will be stored in `mcp.json`:
```json
{
    "servers": {
        "async_cargo_mcp": {
            "type": "stdio",
            "command": "YOUR_PROJECT_PATH/async_cargo_mcp/target/release/async_cargo_mcp",
            "args": []
        }
    },
    "inputs": []
}
```

Restart VSCode to activate the MCP server

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Model Context Protocol (MCP)](https://github.com/modelcontextprotocol) for the protocol specification
- [rmcp](https://github.com/modelcontextprotocol/rmcp) for the Rust MCP implementation
- The Rust community for its excellent async ecosystem

## Features

### Core Cargo Integration
- **Common Cargo Command Support**: build, test, add, remove, check, update, run
- **Parameter Validation**: Type-safe parameter structures with JSON schema validation

### Asynchronous Operation Support
- **Optional Async Notifications**: Enable via `enable_async_notifications` parameter for start/completion events
- **Non-blocking Execution**: LLMs can continue processing while cargo operations run in background
- **Real-time Progress Updates**: Stream operation status and timing information
- **Callback System**: Pluggable callback architecture (NoOp, Logging, Channel-based)

### Operation Monitoring & Management
- **Comprehensive Tracking**: Monitor operation lifecycle from pending → running → completed/failed/cancelled/timed out
- **Timeout Handling**: Configurable timeouts with automatic cancellation of long-running operations
- **Operation Statistics**: Success/failure rates, average duration, detailed metrics
- **Automatic Cleanup**: Background task removes old completed operations
- **Cancellation Support**: Cancel running operations with proper cleanup

### Extensible Architecture
- **Dynamic Command Registry**: Auto-discover available cargo subcommands
- **Trait-based Commands**: Easy extension with `CargoCommand` trait
- **Plugin System**: Register custom cargo command implementations
- **Schema Generation**: Automatic JSON schema for command parameters

### Developer Experience
- **Comprehensive Testing**: 23+ tests covering all functionality
- **Detailed Logging**: Structured logging with operation tracing
- **Error Handling**: Robust error handling with descriptive messages
- **Type Safety**: Full Rust type safety with serde serialization

## Status

### Current Capabilities
- Working directory support for safe testing
- MCP protocol integration with JSON schema validation
- Complete cargo command execution (build, test, run, check, doc, add, remove, update, clean, fix, search, bench, install, tree, version, fetch, rustc, metadata)
- Optional extension commands (clippy, nextest, upgrade, audit, fmt)
- Async callback notifications for progress tracking
- Operation monitoring with timeout and cancellation
- Extensible command registry for auto-discovery
- Comprehensive test suite (55+ tests with robust error handling)

### Upcoming Features
- Enhanced documentation and usage examples
- Integration and testing with popular IDEs and LLM tools (collaborateion and PRs welcome, open an issue)
- RAG documentation to give current API and upstream library API support to the LLM
- Monitor filesystem for LLM changes and preemptively update so future commands return faster

## Installation

Under active development, so only from source at the moment:

```bash
git clone git@github.com:paulirotta/async_cargo_mcp.git
```

## IDE Integration

### VSCode with GitHub Copilot

There are MCP Extensions in the marketplace. They are not necessary and may cause confusion/duplication

First ensure you have enabled VSCode internal MCP server:

```json
    "chat.mcp.enabled": true
```

```bash
cargo build --release
```

In VSCode either add either as Global or Workplace using 
`CTRL/CMD SHIFT P "MCP: Add Server"`

The result is stored in to `mcp.json` as:
```json
{
    "servers": {
        "async_cargo_mcp": {
            "type": "stdio",
            "command": "YOUR_PROJECT_PATH/async_cargo_mcp/target/release/async_cargo_mcp",
            "args": []
        },
    },
    "inputs": []
}
```

Restart VSCode to activate the MCP server

## Architecture

### Core Components

- **`cargo_tools.rs`**: MCP tool implementations for cargo commands
- **`callback_system.rs`**: Async callback infrastructure for progress updates
- **`command_registry.rs`**: Extensible command registration and auto-discovery
- **`operation_monitor.rs`**: Operation lifecycle management and monitoring

### Callback System

Three callback implementations:
- **`NoOpCallbackSender`**: Silent operation (default)
- **`LoggingCallbackSender`**: Logs progress to server console
- **`ChannelCallbackSender`**: Sends updates via async channels

### Operation States

```
Pending → Running → Completed/Failed/Cancelled/TimedOut
```

Each operation tracks:
- Unique ID (UUID)
- Command and description
- Start/end timestamps
- Working directory
- Results and error messages
- Cancellation token

## Testing

Unit and integration tests:

```bash
cargo test
```

A test client is included in the library and used in integration tests

Test cover:
- MCP protocol initialization
- All available cargo commands
- Cargo operations (doc, build, check, add, remove, test, etc.)
- JSON-RPC communication

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Model Context Protocol (MCP)](https://github.com/modelcontextprotocol) for the protocol specification
- [rmcp](https://github.com/modelcontextprotocol/rmcp) for the Rust MCP implementation
- Rust community for excellent async ecosystem

## Development Notes

**MCP Server Usage Experience**: During development and testing, the async_cargo_mcp server performed efficiently for cargo operations. The clean removal of utility commands (say_hello, echo, sum, increment/decrement/get_value) successfully focused the server on its core purpose of providing cargo command access to LLMs. All cargo operations including the full set of core commands (build, test, run, check, doc, add, remove, update, clean, fix, search, bench, install, tree, version, fetch, rustc, metadata) and optional extensions (clippy, nextest, upgrade, audit, fmt) work reliably with proper async notifications and error handling.

## Alternatives

The ecosystem is changing rapidly. Running without an MCP tool but adding some prompt incantations might be the most flexible. In some cases a good tool saves time/money

[jbr's cargo-mcp](https://github.com/jbr/cargo-mcp)

[seemethere's cargo-mcp](https://github.com/seemethere/cargo-mcp)

[SignalWhisperer's cargo-mcp](https://github.com/SignalWhisperer/cargo-mcp)

## Note for Future AI Coding Tools

**Testing Code Changes in VS Code**: When making modifications to this codebase, you can live test your changes directly in VS Code:

1. Run `cargo build --release` to compile your changes
2. Ask the user to restart VS Code so the MCP server reloads with your updated code
3. You can then test your modifications by calling the async-cargo-mcp tools directly in the VS Code environment

This workflow allows for rapid iteration and real-time verification of changes without external setup.