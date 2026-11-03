"""Tiny MCP stdio client for the browser baselines: works with any server command."""
import json, os, subprocess, time


class MCP:
    def __init__(self, cmd, env=None):
        self.p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  bufsize=0, env=dict(os.environ, **(env or {})))
        self.n = 0
        self.rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {},
                                "clientInfo": {"name": "bench", "version": "0"}})
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def send(self, m):
        self.p.stdin.write((json.dumps(m) + "\n").encode())
        self.p.stdin.flush()

    def rpc(self, method, params):
        self.n += 1
        self.send({"jsonrpc": "2.0", "id": self.n, "method": method, "params": params})
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("mcp closed: " + self.p.stderr.read().decode(errors="ignore")[:500])
            try:
                r = json.loads(line)
            except ValueError:
                continue
            if r.get("id") == self.n:
                return r

    def tools(self):
        return self.rpc("tools/list", {})["result"]["tools"]

    def call(self, name, args=None):
        """-> (ms, text, n_images, is_error)"""
        t = time.perf_counter()
        r = self.rpc("tools/call", {"name": name, "arguments": args or {}})
        ms = (time.perf_counter() - t) * 1000
        res = r.get("result", {})
        content = res.get("content", [])
        text = "\n".join(c.get("text", "") for c in content if c.get("type") == "text")
        imgs = sum(1 for c in content if c.get("type") == "image")
        if "error" in r:
            return ms, json.dumps(r["error"]), 0, True
        return ms, text, imgs, bool(res.get("isError"))

    def close(self):
        self.p.kill()
