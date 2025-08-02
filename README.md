# Async Cargo MCP

[![CI](https://github.com/paulirotta/async_cargo_mcp/workflows/Async%20Cargo%20MCP%20Rust/badge.svg)](https://github.com/paulirotta/async_cargo_mcp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

**Model Context Protocol (MCP) server for Cargo with asynchronous response handling and comprehensive operation monitoring.**

This project provides a high performance MCP server that allows Large Language Models (LLMs) to interact with Rust's Cargo build system operations asynchronously. This allows the LLM to proceed on other tasks with the async_cargo_mcp process as an concurrent agent. It supports real-time progress updates, operation cancellation, timeout handling, and extensible command architecture.

## 🚀 Features

### Core Cargo Integration
- **Complete Cargo Command Support**: build, test, add, remove, check, update, run
- **Working Directory Support**: All commands accept optional `working_directory` parameter for safe project isolation
- **Real Command Execution**: Uses actual `cargo` subprocess calls with proper error handling
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

## 🚦 Status

**Phase 2 Complete** - Advanced async callback implementation with operation monitoring

### Current Capabilities
- ✅ Basic cargo command execution (build, test, add, remove, check, update, run)
- ✅ Async callback notifications for progress tracking
- ✅ Operation monitoring with timeout and cancellation
- ✅ Extensible command registry for auto-discovery
- ✅ Comprehensive test suite (23 tests passing)
- ✅ Working directory support for safe testing
- ✅ MCP protocol integration with JSON schema validation

### Upcoming Features
- 🔄 Real-time streaming of cargo output during execution
- 🔄 Advanced operation management UI
- 🔄 Integration with popular IDEs and LLM tools

## 📦 Installation

### Prerequisites
- Rust 1.70+ with Cargo
- Git

### Build from Source
```bash
git clone https://github.com/yourusername/async_cargo_mcp.git
cd async_cargo_mcp
cargo build --release
```

### Command Line Usage
```bash
# Check version
cargo run -- --version

# Get help
cargo run -- --help

# Run MCP server (stdio transport)
cargo run --bin async_cargo_mcp

# Run test client
cargo run --bin client
```

## 🔧 IDE Integration

### VSCode with GitHub Copilot

Add the following to your VSCode settings.json:

```json
{
    "chat.mcp.enabled": true,
    "chat.mcp.discovery.enabled": {
        "async_cargo_mcp": {
            "command": "/YOUR_PATH_TO/async_cargo_mcp/target/release/async_cargo_mcp",
            "args": []
        }
    }
}
```

After building the release version:
```bash
cargo build --release
```

Restart VSCode to activate the MCP server.

## 📖 Usage Examples

### Basic Cargo Operations

```typescript
// Build project
await mcp.callTool("build", {
    working_directory: "/path/to/project"
});

// Add dependency
await mcp.callTool("add", {
    name: "serde",
    version: "1.0",
    features: ["derive"],
    working_directory: "/path/to/project"
});

// Run tests
await mcp.callTool("test", {
    working_directory: "/path/to/project"
});
```

### Async Operations with Notifications

```typescript
// Enable async notifications for long-running builds
await mcp.callTool("build", {
    working_directory: "/path/to/project",
    enable_async_notifications: true
});

// Server will log:
// - Start notification with operation ID
// - Progress updates during execution  
// - Completion notification with timing and results
```

### Advanced Features

```typescript
// Check project without building
await mcp.callTool("check", {
    working_directory: "/path/to/project",
    enable_async_notifications: true
});

// Update all dependencies
await mcp.callTool("update", {
    working_directory: "/path/to/project"
});

// Remove dependency
await mcp.callTool("remove", {
    name: "unused-dep",
    working_directory: "/path/to/project"
});
```

## 🏗️ Architecture

### Core Components

- **`cargo_tools.rs`**: MCP tool implementations for cargo commands
- **`callback_system.rs`**: Async callback infrastructure for progress updates
- **`command_registry.rs`**: Extensible command registration and auto-discovery
- **`operation_monitor.rs`**: Operation lifecycle management and monitoring
- **`streaming_cargo.rs`**: Future streaming implementation for real-time output

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

## 🧪 Testing

Run the comprehensive test suite:

```bash
# All tests
cargo test

# Specific modules
cargo test callback_system
cargo test command_registry
cargo test operation_monitor
cargo test cargo_tools_tests
```

Test coverage includes:
- Unit tests for all modules (15 tests)
- Integration tests with temporary cargo projects (5 tests)
- MCP server functionality tests (3 tests)
- **Total: 23 tests ensuring reliable operation**

## 📊 Monitoring & Metrics

The operation monitor provides detailed statistics:

```rust
let stats = monitor.get_statistics().await;
println!("Success rate: {:.1}%", stats.success_rate());
println!("Average duration: {:?}", stats.average_duration);
println!("Total operations: {}", stats.total);
```

Available metrics:
- Total, pending, running, completed, failed, cancelled, timed out operations
- Success and failure rates
- Average operation duration
- Real-time operation status

## 🔍 Debugging

Enable detailed logging:

```bash
RUST_LOG=debug cargo run --bin async_cargo_mcp
```

Logs include:
- Operation lifecycle events
- Command execution details
- Callback progress updates
- Error diagnostics
- Performance metrics

## 🤝 Contributing

We welcome contributions! Key areas:

1. **New Cargo Commands**: Implement the `CargoCommand` trait
2. **Callback Implementations**: Add new progress notification methods
3. **Output Streaming**: Help implement real-time cargo output streaming
4. **IDE Integrations**: Support for additional editors and LLM tools

### Development Setup

```bash
git clone https://github.com/yourusername/async_cargo_mcp.git
cd async_cargo_mcp
cargo build
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Model Context Protocol (MCP)](https://github.com/modelcontextprotocol) for the protocol specification
- [rmcp](https://github.com/modelcontextprotocol/rmcp) for the Rust MCP implementation
- Rust community for excellent async ecosystem

## VSCode chat test

1. Setup as above and 
2. Select a tool-calling LLM (GPT-4.1..)
3. Type in Github Copilot Chat window:

```bash
@async_cargo_mcp Please increment the counter.
```

## Run the server locally

TODO: We do not yet support HTTP commands in the server

You can instead test by starting it in another window, for example:

```bash
cargo build --release
/YOUR_PATH_TO/async_cargo_mcp/target/release/async_cargo_mcp --spawn=false
```

Send a call_tool request: You need to create a JSON payload representing the tool call. The message must be prefixed with its content length and headers, as per the Language Server Protocol (which MCP's transport layer is based on).

Here is an example of a shell command that constructs and sends a request to increment the counter. You can paste this into a different terminal window.

```bash
# JSON payload for the call_tool request
JSON_PAYLOAD='{"jsonrpc":"2.0","method":"call_tool","params":{"name":"increment","arguments":{}},"id":1}'

# Calculate the content length
CONTENT_LENGTH=$(echo -n "$JSON_PAYLOAD" | wc -c | tr -d ' ')

# Construct the full message with header and send it to the running process
# NOTE: You must run the server first in a separate terminal for this to connect.
printf "Content-Length: %s\r\n\r\n%s" "$CONTENT_LENGTH" "$JSON_PAYLOAD" | /Users/paul/github/async_cargo_mcp/target/release/async_cargo_mcp --spawn=false
```

This command will send the request and the server will print its JSON-RPC response to stdout. This manual method is useful for debugging the raw protocol but testing through the VS Code Chat view is much more convenient for general use.