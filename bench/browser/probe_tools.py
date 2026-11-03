"""Lists the tools of an MCP server and the size of its tool schemas: python probe_tools.py <server command...>"""
import json, sys
from mcp import MCP

m = MCP(sys.argv[1:])
ts = m.tools()
print(len(ts), "tools; schema chars:", len(json.dumps(ts)))
for t in ts:
    print(" -", t["name"], list((t.get("inputSchema") or {}).get("properties", {})))
m.close()
