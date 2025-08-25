# Product Specification: async_cargo_mcp

## 1. Introduction

`async_cargo_mcp` is a Model Context Protocol (MCP) server that enables AI assistants to execute Rust's `cargo` commands asynchronously and efficiently. It serves as a bridge between AI agents (like GitHub Copilot in VS Code) and Rust development environments, providing true concurrent task execution.

### Key Capabilities

- **Asynchronous Execution**: AI can initiate long-running cargo tasks (`build`, `test`, `clippy`) and continue with other work while commands run in the background
- **High Performance**: Pre-warmed shell pools minimize command startup latency for near-instantaneous cargo operations
- **Safety & Security**: Structured, validated interfaces prevent arbitrary shell execution with working directory isolation
- **Professional AI Communication**: Clear feedback through progress notifications and standardized tool hints designed for AI consumption
- **Flexible Control**: Configurable command availability with synchronous/asynchronous execution modes

**Status**: Production-ready as of August 2025. All core functionality implemented and tested.

## 2. Architecture Overview

The server employs a modular, high-performance architecture optimized for concurrent AI workflows:

**Core Components:**

- **MCP Server Interface**: JSON-RPC 2.0 communication over stdin/stdout
- **Cargo Tools Router**: Validates requests and routes to appropriate cargo command implementations
- **Shell Pool Manager**: Maintains pre-warmed shell processes for optimal performance
- **Operation Monitor**: Tracks async operation lifecycle and results storage
- **Callback System**: Delivers real-time progress updates via `$/progress` notifications

**Data Flow:**

1. AI sends tool request → Server validates and generates operation ID
2. Background task executes cargo command via shell pool
3. Results automatically pushed to AI via progress notifications
4. AI continues concurrent work, optionally queries status or waits for specific results

## 3. Command Line Interface

```bash
async_cargo_mcp [OPTIONS]

Options:
  --enable-wait          Enable legacy wait tool (default: disabled)
  --synchronous          Force all operations to run synchronously
  --disable-shell-pools  Disable performance-optimized shell pooling
  --timeout <SECONDS>    Set operation timeout (default: 300)
  --disable-tools <LIST> Comma-separated list of tools to disable
  --help                 Print help information
```

**Recommended Production Usage:**

```bash
# Default configuration (automatic result push)
async_cargo_mcp

# Legacy compatibility mode
async_cargo_mcp --enable-wait

# Synchronous mode for debugging
async_cargo_mcp --synchronous
```

## 4. Available Tools

### Core Cargo Commands

- `build` - Compile packages with configurable features and targets
- `test` / `nextest` - Run test suites (nextest preferred for speed)
- `check` - Fast compile validation without artifacts
- `clippy` - Enhanced linting with auto-fix capabilities
- `fmt` - Code formatting with rustfmt
- `doc` - Generate documentation
- `run` - Execute binaries with arguments

### Dependency Management

- `add` - Add dependencies with version and feature control
- `remove` - Remove dependencies safely
- `update` - Update to latest compatible versions
- `upgrade` - Upgrade to latest versions (requires cargo-edit)
- `fetch` - Download dependencies without building

### Project Tools

- `clean` - Remove build artifacts
- `tree` - Display dependency trees
- `search` - Search crates.io
- `audit` - Security vulnerability scanning
- `install` - Global tool installation

### Operation Management

- `status` - Non-blocking query of running operations
- `wait` - Legacy tool for explicit result retrieval (disabled by default)
- `sleep` - Testing utility for timeout scenarios

## 5. AI Integration Patterns

### Concurrent Workflow (Recommended)

```json
// 1. Start long operation
{
  "name": "build",
  "arguments": {
    "working_directory": "/project",
    "enable_async_notification": true
  }
}

// 2. Continue with other tasks immediately
{
  "name": "clippy",
  "arguments": {
    "working_directory": "/project",
    "enable_async_notification": true
  }
}

// 3. Results delivered automatically via $/progress notifications
```

### Status Monitoring

```json
{
  "name": "status",
  "arguments": {
    "working_directory": "/project"
  }
}
// Returns: JSON list of active operations with runtime and status
```

### Legacy Synchronous Mode

```json
{
  "name": "build",
  "arguments": {
    "working_directory": "/project"
    // enable_async_notification not specified = synchronous
  }
}
```

## 6. Performance Features

### Shell Pool System

- Pre-warmed bash processes per working directory
- Eliminates process startup overhead (typically 50-200ms savings per command)
- Configurable pool sizes with automatic health monitoring
- Graceful cleanup of idle shells

### Concurrency Metrics

- Tracks "concurrency gap" between operation dispatch and result consumption
- Efficiency scoring for AI task parallelism effectiveness
- Performance analytics for optimization feedback

### Resource Management

- Configurable operation timeouts with graceful cancellation
- Memory-efficient result storage with automatic cleanup
- Working directory isolation prevents cross-contamination

## 7. AI Behavior Guidance

### Automatic Result Push System

By default, the server automatically pushes operation results via `$/progress` notifications, eliminating the need for explicit wait calls and enabling true concurrent AI behavior.

### Anti-Pattern Prevention

- Status polling detection with guidance after 3+ consecutive calls
- Professional, emoji-free messages designed for AI consumption
- Clear recommendations for optimal tool usage patterns

### Tool Hints & Education

Each tool response includes contextual guidance:

- When to use async vs synchronous execution
- Optimal concurrency patterns for multi-step workflows
- Performance recommendations based on operation type

## 8. Security & Safety

### Input Validation

- Structured request schemas prevent malicious command injection
- Working directory validation ensures operations stay within intended scope
- Command argument sanitization and validation

### Process Isolation

- Commands executed in isolated shell environments
- No arbitrary shell command execution capabilities
- Configurable tool disabling for restricted environments

### Resource Protection

- Configurable timeouts prevent runaway operations
- Memory-bounded result storage with cleanup policies
- Rate limiting on status queries to prevent resource abuse

## 9. Production Deployment

### Requirements

- Rust toolchain with cargo, clippy, and rustfmt
- Optional: cargo-nextest for faster testing
- Optional: cargo-edit for enhanced dependency management
- Optional: cargo-audit for security scanning

### Integration Examples

**VS Code MCP Client:**

```json
{
  "mcpServers": {
    "async_cargo_mcp": {
      "command": "/path/to/async_cargo_mcp",
      "args": []
    }
  }
}
```

**Custom AI Integration:**

```javascript
// Initialize MCP client connection
const client = new MCPClient();
await client.connect({
  command: "async_cargo_mcp",
  args: ["--enable-wait"], // if legacy compatibility needed
});

// Execute cargo commands with automatic result handling
await client.callTool("build", {
  working_directory: projectPath,
  enable_async_notification: true,
});
```

### Monitoring & Logging

- Structured JSON logging with configurable levels
- Operation lifecycle tracking and metrics
- Performance monitoring with concurrency analytics
- Health status reporting for shell pool management

## 10. Future Enhancements

### Ecosystem Expansion Opportunities

- **IDE Integration**: Visual operation tracker for VS Code with actionable notifications
- **AI Personas**: "Concurrent Executor" prompt engineering for optimal parallelism
- **Enhanced Callbacks**: Rich progress updates with build artifact information
- **Cross-Platform**: Windows PowerShell support alongside Unix shell pools

### Advanced Features Under Consideration

- Remote cargo execution for distributed development
- Build artifact caching and intelligent invalidation
- Integration with Rust language server for enhanced diagnostics
- Custom workflow automation with operation chaining

The async_cargo_mcp server provides a solid foundation for these future enhancements while delivering immediate value through its current production-ready feature set.
