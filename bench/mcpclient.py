import os
import json, subprocess, time, sys

CUA = os.environ.get("CUA_DRIVER", os.path.join(os.environ.get("LOCALAPPDATA", ""), "Programs", "Cua", "cua-driver", "bin", "cua-driver.exe"))


class MCP:
    def __init__(self):
        self.p = subprocess.Popen([CUA, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, bufsize=0)
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
                raise RuntimeError("mcp closed")
            r = json.loads(line)
            if r.get("id") == self.n:
                return r

    def call(self, name, args=None):
        t = time.perf_counter()
        r = self.rpc("tools/call", {"name": name, "arguments": args or {}})
        ms = (time.perf_counter() - t) * 1000
        return ms, r

    def close(self):
        self.p.terminate()


def text(r):
    res = r.get("result", {})
    return "\n".join(c.get("text", "") for c in res.get("content", []) if c.get("type") == "text")


if __name__ == "__main__":
    m = MCP()
    ms, r = m.call("launch_app", {"name": "notepad"})
    print("launch_app", round(ms), "ms")
    print(text(r)[:800])
    ms, r = m.call("list_windows")
    print("list_windows", round(ms), "ms")
    print(text(r)[:1500])
    m.close()
