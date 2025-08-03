#!/bin/bash

# Test script for async_cargo_mcp MCP server
echo "=== Testing MCP Server ==="

# Build release version first
echo "Building release version..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo

# Start the server and send proper MCP messages
echo "Testing MCP protocol interactions..."
{
    # 1. Initialize
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
    
    # 2. Initialized notification (required after initialize)
    echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
    
    # 3. Call tools/list to see available tools
    echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    
    # 4. Test the increment tool
    echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"increment","arguments":{}}}'
    
    # 5. Test get_value
    echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_value","arguments":{}}}'
    
    # 6. Test echo with arguments
    echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello MCP!"}}}'
    
    # 7. Test sum
    echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"sum","arguments":{"a":5,"b":3}}}'
    
    # 8. Test a cargo command (build in current directory)
    echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"check","arguments":{}}}'
    
} | /Users/paul/github/async_cargo_mcp/target/release/async_cargo_mcp 2>/dev/null

echo
echo "=== Test Complete ==="
