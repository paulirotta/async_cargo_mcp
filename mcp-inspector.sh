#!/bin/bash

# Print current directory message
echo "Starting MCP Inspector from directory: $(pwd)"

# Kill any existing MCP inspector processes
echo "Checking for existing MCP inspector processes..."
if pgrep -f "@modelcontextprotocol/inspector" > /dev/null; then
    echo "Killing existing MCP inspector processes..."
    pkill -f "@modelcontextprotocol/inspector"
    sleep 2  # Give processes time to terminate
    echo "Existing processes terminated."
else
    echo "No existing MCP inspector processes found."
fi

# Build the project in release mode
echo "Building Rust project with cargo build --release..."
cargo build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful! Launching MCP Inspector..."
    echo "You can now interact directly with your MCP server tools."
    
    # Run the MCP inspector
    npx @modelcontextprotocol/inspector /Users/paul/github/async_cargo_mcp/target/release/async_cargo_mcp
else
    echo "Build failed! Please check the build errors above."
    exit 1
fi
