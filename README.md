# async_cargo_mcp
Run Rust Cargo commands with optional asynchonous callback using Model Context Protocol (MCP). This allows a Large Langage Model (LLM) to continue to think and run multiple MCP commands concurrently.

### This project is early stage and in active development

## Command line checks

```bash
cargo run -- --version
cargo run -- --help
```

## Setup in VSCode Copilot

Add the following

```json
    "chat.mcp.enabled": true,
    "chat.mcp.discovery.enabled": {
        "sync_cargo_mcp": {
            "command": "/YOUR_PATH_TO/async_cargo_mcp/target/release/async_cargo_mcp",
            "args": [
                "--spawn=false"
            ]
        },
        "async_cargo_mcp": {
            "command": "/YOUR_PATH_TO/async_cargo_mcp/target/release/async_cargo_mcp",
            "args": [
                "--spawn=true"
            ]
        }
    },
```

Restart VSCode after you (re)build it.

```bash
cargo build --release
```

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