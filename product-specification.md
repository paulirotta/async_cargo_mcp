# Product Specification: async_cargo_mcp

## 1. Introduction

`async_cargo_mcp` is a Model Context Protocol (MCP) server designed to allow AI assistants to execute Rust's `cargo` commands in a safe, efficient, and asynchronous manner. It acts as a bridge between an AI agent (like GitHub Copilot in VSCode) and the user's Rust development environment.

The primary goals of this project are:

- **Concurrency**: Enable the AI to initiate long-running `cargo` tasks (like `build`, `test`, `clippy`) and continue with other tasks (e.g., planning, writing code, updating documentation) while the `cargo` command runs in the background. This significantly improves the overall task completion speed.
- **Performance**: Utilize a pre-warmed shell pool to minimize command startup latency, providing a near-instantaneous feel for `cargo` operations.
- **Safety**: Provide a structured and validated interface for `cargo` commands, preventing arbitrary shell execution and ensuring operations are confined to the intended working directory.
- **Usability**: Offer clear feedback to the AI through progress notifications and detailed results, including standardized tool hints that guide the AI on how to use the asynchronous features effectively.
- **Control**: Give the user/operator control over which `cargo` commands are available and how they behave (e.g., synchronous vs. asynchronous execution).

This document provides a complete specification of the `async_cargo_mcp` server, including its architecture, communication protocol, command-line interface, and implementation details.

## 2. Architecture

The server is built with a modular architecture, with each component having a distinct responsibility.

- **`main.rs` (Server Entry Point)**:

  - Parses command-line arguments using `clap`.
  - Initializes the logging system (`tracing`).
  - Creates and configures the `OperationMonitor` for tracking all tasks.
  - Creates and configures the `ShellPoolManager` for high-performance command execution.
  - Instantiates the `AsyncCargo` service.
  - Starts the MCP server, listening for requests on standard I/O.

- **`cargo_tools.rs` (Core Logic & Tool Router)**:

  - Defines the `AsyncCargo` struct, which holds the state of the service.
  - Uses the `rmcp::tool_router` macro to define all available tools (e.g., `build`, `test`, `add`).
  - Each tool handler function is responsible for:
    1.  Parsing and validating the incoming request parameters.
    2.  Deciding whether to execute the command synchronously or asynchronously based on the `enable_async_notification` parameter and the server's `--synchronous` flag.
    3.  For async operations:
        - Generating a unique `operation_id`.
        - Registering the operation with the `OperationMonitor`.
        - Spawning a background `tokio` task to perform the actual work.
        - Returning an immediate response to the AI with the `operation_id` and a tool hint.
    4.  For sync operations:
        - Executing the command directly.
        - Waiting for the result.
        - Returning the complete result to the AI.
  - Contains the `..._implementation` functions that construct the `cargo` command string from the request parameters and execute it via the `ShellPoolManager`.

- **`shell_pool.rs` (Performance Layer)**:

  - Implements the high-performance shell pooling system.
  - `ShellPoolManager`: Manages a collection of `ShellPool`s, one for each working directory.
  - `ShellPool`: Contains a set of `PrewarmedShell` processes for a specific directory.
  - `PrewarmedShell`: A long-lived `bash` process that is initialized with a JSON-based communication protocol. It can execute commands without the overhead of spawning a new process each time.
  - Includes background tasks for health checking and cleaning up idle shells.

- **`operation_monitor.rs` (State Management)**:

  - Tracks the lifecycle of every operation (`Pending`, `Running`, `Completed`, `Failed`, `Cancelled`, `TimedOut`).
  - Stores the results of completed operations so they can be retrieved later by the `wait` command.
  - Handles operation timeouts.
  - Provides a mechanism to cancel running operations.

- **`callback_system.rs` & `mcp_callback.rs` (Communication)**:
  - `callback_system.rs`: Defines a generic `CallbackSender` trait for sending progress updates.
  - `mcp_callback.rs`: Implements the `CallbackSender` trait to send `$/progress` notifications to the MCP client, allowing the AI to receive real-time feedback on running operations.

### Data Flow (Asynchronous Operation)

1.  **AI Client (VSCode)** sends a `call-tool` request (e.g., `build`) to the server via `stdin`.
2.  **`main.rs`** pipes the request to the `AsyncCargo` service.
3.  **`cargo_tools.rs`**'s `build` handler is invoked.
4.  It determines the operation is async, generates `op_build_123`, and registers it with the **`OperationMonitor`**.
5.  It immediately sends a `call-tool` response back to the client with the `operation_id`.
6.  Simultaneously, it spawns a `tokio` task.
7.  The background task requests a shell from the **`ShellPoolManager`** for the specified `working_directory`.
8.  The **`ShellPoolManager`** provides a `PrewarmedShell`.
9.  The task sends the `cargo build ...` command to the shell.
10. The shell executes the command and captures `stdout` and `stderr`.
11. When the command completes, the shell sends the result back to the task.
12. The task calls `complete_operation` on the **`OperationMonitor`**, storing the result.
13. The AI, having received the initial response, continues its work. Later, it sends a `wait` request with `operation_ids: ["op_build_123"]`.
14. The `wait` handler in **`cargo_tools.rs`** calls `wait_for_operation` on the **`OperationMonitor`**.
15. The **`OperationMonitor`** retrieves the stored result for `op_build_123` and returns it.
16. The `wait` handler sends the final, detailed result back to the AI client.

## 3. Communication Protocol (MCP)

The server communicates using the [Model Context Protocol (MCP) 2.0](https://microsoft.github.io/language-server-protocol/specifications/mcp/2.0/specification/). All communication happens over `stdin` and `stdout` using JSON-RPC 2.0 messages.

### Example: Asynchronous `build` and `wait`

This example shows the full sequence for running `cargo build` asynchronously and then retrieving the result.

**1. Client -> Server: `build` request**

The AI decides to build the project.

```json
{
  "jsonrpc": "2.0",
  "method": "call-tool",
  "params": {
    "name": "build",
    "arguments": {
      "working_directory": "/Users/paul/github/async_cargo_mcp",
      "enable_async_notification": true
    }
  },
  "id": 1
}
```

**2. Server -> Client: `build` response (immediate)**

The server immediately acknowledges the request and provides an `operation_id`.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Build operation op_build_123 started at 14:35:01 in the background.\\n\\n### ASYNC CARGO OPERATION: build (ID: op_build_123)\\n..."
      }
    ]
  }
}
```

_(The full tool hint is omitted for brevity)_

**3. Server -> Client: Progress Notification (optional)**

While the build is running, the server can send progress updates.

```json
{
  "jsonrpc": "2.0",
  "method": "$/progress",
  "params": {
    "token": "op_build_123",
    "value": {
      "kind": "report",
      "message": "Compiling crate-a v0.1.0"
    }
  }
}
```

**4. Client -> Server: `wait` request**

After performing other actions, the AI is now ready for the build result.

```json
{
  "jsonrpc": "2.0",
  "method": "call-tool",
  "params": {
    "name": "wait",
    "arguments": {
      "operation_ids": ["op_build_123"]
    }
  },
  "id": 2
}
```

**5. Server -> Client: `wait` response (final result)**

The server retrieves the completed operation's result and sends it back.

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Wait completed. Longest duration: 5 seconds."
      },
      {
        "type": "text",
        "text": "OPERATION COMPLETED: 'op_build_123'\\nCommand: cargo build\\nDescription: Building project in the background\\nWorking Directory: /Users/paul/github/async_cargo_mcp\\n\\n=== FULL OUTPUT ===\\n+ Build completed successfully in /Users/paul/github/async_cargo_mcp.\\nOutput:    Finished dev [unoptimized + debuginfo] target(s) in 4.50s"
      }
    ]
  }
}
```

### Example: Synchronous `check`

This example shows a synchronous command where the result is returned immediately.

**1. Client -> Server: `check` request**

```json
{
  "jsonrpc": "2.0",
  "method": "call-tool",
  "params": {
    "name": "check",
    "arguments": {
      "working_directory": "/Users/paul/github/async_cargo_mcp"
      // "enable_async_notification" is omitted or false
    }
  },
  "id": 3
}
```

**2. Server -> Client: `check` response (final result)**

The server blocks until `cargo check` completes and returns the full result in one go.

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "+ Check completed successfully in /Users/paul/github/async_cargo_mcp.\\nOutput:    Finished dev [unoptimized + debuginfo] target(s) in 1.20s"
      }
    ]
  }
}
```

### Special Case: `nextest` and Early Lock Release

The `nextest` command is known for releasing the `Cargo.lock` file early, before all tests have finished running. The `async_cargo_mcp` server's architecture handles this gracefully.

Because each command runs in its own isolated shell process (managed by the `ShellPool`), one `nextest` operation releasing its lock does not interfere with other pending or subsequent `cargo` commands. If the AI queues `nextest` followed by `clippy`, the `clippy` command will be able to acquire the `cargo` lock as soon as `nextest` releases it, even while the tests themselves are still running to completion in the background. This further enhances the potential for concurrent execution.

## 4. Command Line Interface

The server is configured via command-line arguments.

```
Usage: async_cargo_mcp [OPTIONS]

Options:
      --timeout <SECONDS>
          Set default timeout in seconds for cargo operations (default: 300)
      --shell-pool-size <COUNT>
          Number of pre-warmed shells per working directory for faster command execution
      --max-shells <COUNT>
          Maximum total number of shells across all working directories
      --disable-shell-pools
          Disable shell pools and use direct tokio::process::Command spawning
      --synchronous
          Force synchronous execution of all operations, disabling async callbacks and notifications
      --log-to-file
          Write logs to a rolling daily file instead of stderr
      --verbose
          Enable verbose debug logging
      --disable <TOOL>
          Disable specific tools by name. Accepts comma-separated list or repeat flag. Example: --disable build,test,clippy --disable audit
  -h, --help
          Print help
  -V, --version
          Print version
```

## 5. Implementation Details

- **Language**: Rust
- **Core Framework**: `tokio` for asynchronous I/O and task management.
- **MCP Library**: `rmcp` (official Rust MCP SDK).
- **Argument Parsing**: `clap`.
- **Logging**: `tracing`.
- **Serialization**: `serde` and `serde_json`.

### Key Data Structures

- **`AsyncCargo`**: The main struct holding the server state, including the `OperationMonitor` and `ShellPoolManager`.
- **`OperationInfo`**: A struct in `operation_monitor.rs` that stores all information about a single operation, including its ID, command, state, timings, and result.
- **`PrewarmedShell`**: A struct in `shell_pool.rs` representing a single, persistent shell process. It manages the `stdin` and `stdout` for communication.
- **Request Structs**: Each `cargo` command has a corresponding request struct (e.g., `BuildRequest`, `TestRequest`) defined in `cargo_tools.rs`. These structs use `serde::Deserialize` and `schemars::JsonSchema` to automatically handle request parsing and schema generation for the MCP client.

## 6. Future Plans & Strategic Improvements

### Analysis of AI Concurrency (`Real vs. Desired`)

**Desired State:** The ultimate goal is for the AI to achieve maximum "task parallelism". This means the AI should:

1.  Identify a long-running `cargo` task.
2.  Dispatch it asynchronously.
3.  Immediately pivot to a completely different but productive task that does not depend on the result of the `cargo` command (e.g., writing documentation, refactoring unrelated code, planning the next major step).
4.  Only `wait` for the result at the last possible moment when it's required to proceed.
    The "cargo AND AI active at the same time" percentage should be high, approaching the total duration of the `cargo` tasks.

**Current Reality:** The actual usage pattern observed in many AI models is more sequential, even with the async capability. The AI often does this:

1.  Dispatches an async `cargo` task.
2.  Writes a message like, "Okay, the tests are running. I will now wait for them to complete."
3.  Immediately calls `wait`.

This pattern negates the primary benefit of the asynchronous architecture. The "cargo AND AI active at the same time" percentage is near zero.

**Why does this happen?**

- **Simplicity of State Management:** For an AI, managing a simple, linear sequence of tasks is easier than juggling multiple concurrent operations and their dependencies. Thinking in parallel is a complex cognitive task.
- **Instruction Following:** AIs are trained to follow instructions meticulously. If a user says "run the tests and then fix the bug," the AI interprets this as a strict sequence. The current tool hints, while good, might not be strong enough to override this fundamental behavior.
- **Lack of "Filler" Tasks:** The AI might not have an obvious, independent task to work on while waiting. It might not be instructed to, for example, "write the documentation for function X while the tests for function Y are running."
- **Prompt Engineering:** The prompts given to the AI often imply a sequential workflow. The "Rust Beast Mode" persona helps, but the core user request drives the AI's behavior.

### Systematic Improvements for Concurrency

To bridge the gap between the current and desired state of AI concurrency, a multi-pronged strategy is required, focusing on tooling, AI guidance, and client-side enhancements.

1.  **Server-Side Enhancements & Tooling:**

    - **Default to Pushed Results:** The server's default behavior should be to push final results via `$/progress` notifications. The `wait` tool should be treated as a legacy feature for specific use cases, not the primary method for retrieving results.
    - **Introduce an Optional `wait` Mode:** Add a server flag like `--enable-wait`. Only when this flag is active would the `wait` tool function as it does now. Otherwise, calling `wait` would return a hint explaining that results are pushed automatically. This forces a paradigm shift in AI behavior.
    - **Deprecate the `wait` tool:** The tool's description should be changed to strongly discourage its use, marking it as a legacy tool for debugging or for use in an explicit wait-enabled mode. The primary method for receiving results should be the automatic push of the final result via a `$/progress` notification.
    - **Add a `status` Tool:** Introduce a new, lightweight `status` tool. This would allow the AI to non-blockingly query the state of all ongoing operations (`op_id`, `command`, `status: running/queued`, `runtime`). This provides visibility without forcing a wait, enabling more intelligent planning.
    - **Intelligent "Nudge" Responses:** If `wait` is called (in a mode where it's allowed but discouraged), and the call is made very soon after the async task was started, the server can return the result with a helpful hint: _"Hint: You requested this result almost immediately. To work more efficiently, perform other tasks and rely on the result being pushed to you automatically."_
    - **Concurrency Metrics:** The server could track the "concurrency gap"—the time between an async call and its corresponding `wait` call. This metric could be logged or optionally returned with results to provide feedback on how effectively the AI is parallelizing tasks, helping developers refine prompts and AI personas.

2.  **Advanced Prompt Engineering & AI Personas:**

    - **"Concurrent Executor" Persona:** Instruct the AI to adopt a persona focused on minimizing wall-clock time. Key instructions would include: _"Your primary goal is to maximize concurrency. Always look for opportunities to dispatch a `cargo` command and then immediately proceed with unrelated coding, planning, or documentation tasks. You must rely on pushed notifications for results and avoid using the `wait` command unless absolutely necessary."_
    - **Educate on Explicit Prompting:** Guide users to structure their prompts to encourage parallelism. Instead of a sequential "Run tests, then refactor," a better prompt is: "Start the full test suite in the background. While it runs, begin refactoring the `parser::parse` function."

3.  **IDE and Client-Side Enhancements:**

    - **Visual Operation Tracker:** The VS Code client could feature a non-intrusive UI element (e.g., in the status bar) that lists running background operations. This gives both the user and the AI a constant, shared visual context of concurrent tasks.
    - **Actionable Result Notifications:** The client-side integration must ensure that `$/progress` notifications containing final, complete results are clearly presented to the AI model. The format should be unambiguous, allowing the AI to easily parse the result and understand that the specific operation is finished.

## 7. Other Suggestions for Improvement

- **Interactive `cargo watch`:** Implement a tool to start `cargo watch` as a long-running background process. The AI could then query its status or view its latest output without needing to `wait` for it to terminate.
- **Profile-Guided Optimization (PGO) Support:** Add tools to streamline the process of running a benchmark with PGO, which is a multi-step process that could be abstracted into a single, powerful async tool.
- **Cross-Compilation Management:** Simplify cross-compilation by adding a tool that checks for and installs required `rustup` targets before attempting a `--target` build.
- **Enhanced `cargo add`:** Allow specifying features when adding a new dependency, e.g., `add --name serde --features derive`.
- **Workspace-Wide Commands:** While some commands support `--workspace`, it could be beneficial to have more explicit workspace-level tools that operate on all members of a workspace, perhaps with a selection mechanism.
