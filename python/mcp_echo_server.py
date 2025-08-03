#!/usr/bin/env python3
import json
import sys
import time

def main():
    with open("/tmp/mcp_echo_server.log", "a") as f:
        f.write(f"Started at {time.time()}\n")
    print('{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{},"logging":{}},"serverInfo":{"name":"mcp-echo-server","version":"1.0.0"}}}', file=sys.stdout, flush=True)
    
    for line in sys.stdin:
        try:
            msg = json.loads(line.strip())
            if msg.get('method') == 'tools/call':
                with open("/tmp/mcp_echo_server.log", "a") as f:
                    f.write(f"Received tools/call at {time.time()}\n")
                response = {
                    "jsonrpc": "2.0",
                    "id": msg.get('id'),
                    "result": {
                        "content": [{"type": "text", "text": "Hello from simple MCP!"}]
                    }
                }
                print(json.dumps(response), file=sys.stdout, flush=True)
        except:
            pass

if __name__ == "__main__":
    main()
    