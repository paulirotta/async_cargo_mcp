#!/bin/bash

# Test script for async_cargo_mcp MCP server
echo "=== Testing MCP Server ==="

# Start the server and send proper MCP messages
{
    # 1. Initialize
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
    
    # 2. Initialized notification (required after initialize)
    echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
    
    # 3. Call tools/list to see available tools
    echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    
    # 4. Call the increment tool
    echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"increment","arguments":{}}}'
    
} | /Users/paul/github/async_cargo_mcp/target/release/async_cargo_mcp

echo "=== Test Complete ==="
