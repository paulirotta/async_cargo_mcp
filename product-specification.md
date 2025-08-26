# Product Specification: async_cargo_mcp

**Version:** 2.0
**Date:** August 26, 2025
**Status:** Production Ready

## 1. Introduction

`async_cargo_mcp` is a Model Context Protocol (MCP) server that enables AI assistants to execute Rust's `cargo` commands asynchronously and efficiently. It serves as a high-performance bridge between AI agents (like GitHub Copilot) and Rust development environments, enabling true concurrent task execution.

### Key Capabilities

- **Asynchronous Execution**: AI can initiate long-running `cargo` tasks (`build`, `test`, `clippy`) and immediately continue with other work while commands run in the background.
- **High Performance**: A pre-warmed shell pool system minimizes command startup latency from ~150ms to under 20ms.
- **Safety & Security**: Structured, validated interfaces prevent arbitrary shell execution, with strict working directory isolation.
- **Professional AI Guidance**: Clear feedback through automatic progress notifications and standardized tool hints designed for optimal AI consumption and anti-pattern prevention.
- **Flexible Control**: Configurable command availability with distinct synchronous and asynchronous execution modes.

## 2. Architecture Overview

The server employs a modular, high-performance architecture optimized for concurrent AI workflows.

- **Core Components**:

  - **MCP Server Interface**: JSON-RPC 2.0 communication over stdin/stdout.
  - **Cargo Tools Router**: Validates requests and routes them to the correct `cargo` command implementations.
  - **Shell Pool Manager**: Maintains pre-warmed shell processes for each working directory to eliminate startup overhead.
  - **Operation Monitor**: Tracks the lifecycle of all asynchronous operations (e.g., `running`, `completed`, `timed-out`).
  - **Callback System**: Delivers real-time progress updates and final results automatically via `$/progress` notifications.

- **Data Flow (Async Operation)**:
  1. AI sends a tool request (e.g., `build` with `enable_async_notification: true`).
  2. The server validates the request, generates a unique `operation_id`, and immediately returns it to the AI.
  3. A background task executes the `cargo` command using a shell from the pool.
  4. The AI, unblocked, proceeds with other tasks (planning, coding, etc.).
  5. Upon completion, the server automatically pushes the final result to the AI via a `$/progress` notification.

## 3. Command Line Interface

```bash
async_cargo_mcp [OPTIONS]

Options:
  --synchronous          Force all operations to run synchronously, blocking until complete.
                         (Disables async notifications and the 'wait' tool).
  --disable-shell-pools  Disable the performance-optimized shell pooling.
  --timeout <SECONDS>    Set the global operation timeout (default: 300).
  --disable-tools <LIST> Comma-separated list of tools to disable (e.g., "add,remove").
  --help                 Print help information.
```

## 4. Available Tools

### Core Cargo Commands

- `build`, `check`, `clippy`, `doc`, `fmt`, `run`, `test`, `nextest`

### Dependency Management

- `add`, `remove`, `update`, `upgrade`, `fetch`

### Project & Info

- `clean`, `tree`, `search`, `audit`, `install`, `metadata`, `version`, `rustc`

### Operation Management

- **`status`**: Non-blockingly query the status of running operations.
- **`wait`**: Wait for one or more async operations to complete. (Available in async mode only; its use is discouraged in favor of automatic result pushes).
- **`sleep`**: A utility for testing timeout scenarios.

## 5. AI Integration & Behavior

### Execution Modes

- **Async Mode (Default)**: Operations start instantly and return an `operation_id`. Results are pushed automatically. The AI should continue working on other tasks.
- **Synchronous Mode (`--synchronous`)**: Each operation blocks until it completes and returns the full result directly. `wait` is not available.

### AI Guidance & Anti-Pattern Prevention

The server is designed to guide the AI toward efficient, parallel workflows:

- **Automatic Results**: The primary mechanism is automatic result pushing, freeing the AI from needing to `wait`.
- **Status Polling Detection**: If the AI calls `status` repeatedly for the same operation, the server provides a hint suggesting more efficient patterns.
- **Clear Tool Hints**: All responses include professional, emoji-free guidance on optimal next steps, encouraging concurrency.

## 6. Future Enhancements & Roadmap

This section outlines prioritized ideas for improving the server and the AI's interaction with it.

### Tier 1: Core Experience & AI Behavior

- **Visual Operation Tracker (IDE Integration)**: A VS Code extension or UI element that provides a visual list of running background operations. This would give both the user and the AI a shared, persistent context of concurrent tasks.
- **Advanced AI Personas & Prompting**: Develop and document advanced "Concurrent Executor" personas that explicitly instruct the AI to maximize task parallelism and avoid waiting.
- **Interactive `cargo watch`**: Implement a tool to start `cargo watch` as a long-running background process. The AI could then query its status or view its latest output without needing to `wait` for it to terminate.

### Tier 2: Advanced Cargo Workflows

- **Profile-Guided Optimization (PGO) Support**: Abstract the multi-step PGO process into a single, powerful async tool.
- **Cross-Compilation Management**: Simplify cross-compilation by adding a tool that checks for and installs required `rustup` targets before a `--target` build.
- **Enhanced Dependency Management**: Allow specifying features when adding a dependency (e.g., `add --name serde --features derive`).

### Tier 3: Ecosystem & Enterprise

- **Remote Cargo Execution**: Support for running `cargo` commands on a remote or containerized development environment.
- **Build Artifact Caching**: Intelligent caching and invalidation of build artifacts across different operations.
- **Windows PowerShell Support**: Add a `PowerShell` pool for first-class support on Windows.
