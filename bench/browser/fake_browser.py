"""End-to-end check of the native host and the MCP tool without a real browser.
Plays the role of Chrome (starts `fastuse chrome-extension://...` and speaks native messaging), then
asks a normal fastuse MCP server for browser_state and checks the answer.
Usage: python fake_browser.py <path-to-fastuse.exe>"""
import json, os, struct, subprocess, sys, threading, time
from mcp import MCP

exe = sys.argv[1]
ORIGIN = "chrome-extension://andnpcpahlilpjlipaogpekbnljedfcb/"
host = subprocess.Popen([exe, ORIGIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
seen = []


def send(v):
    b = json.dumps(v).encode()
    host.stdin.write(struct.pack("<I", len(b)) + b)
    host.stdin.flush()


def recv():
    h = host.stdout.read(4)
    if len(h) < 4:
        return None
    return json.loads(host.stdout.read(struct.unpack("<I", h)[0]))


def fake_extension():
    send({"event": "hello", "browser": "FakeBrave", "extension": "0.0.1", "id": "andnpcpahlilpjlipaogpekbnljedfcb"})
    while True:
        m = recv()
        if m is None:
            return
        seen.append(m["method"])
        if m["method"] == "tabs.list":
            r = {"tabs": [{"id": 5, "title": "Test page", "url": "https://example.test/", "active": True}], "hidden": 2}
        else:
            r = {"tab": {"id": 5}, "page": {"title": "Test page", "url": "https://example.test/", "viewport": {"w": 1000, "h": 700, "scrollY": 0, "pageH": 900},
                 "count": 1, "rows": 2, "truncated": False, "ms": 1.5, "text": "- Heading[1] \"Hello\"\n- [0] Button \"Go\" @(10,10,50,20)"}}
        r["browser"] = {"name": "FakeBrave", "extension": "0.0.1"}
        send({"id": m["id"], "result": r})


threading.Thread(target=fake_extension, daemon=True).start()
time.sleep(1.0)

m = MCP([exe])
ms, text, _, is_err = m.call("browser_state", {})
print(f"first browser_state: {ms:.1f} ms, error={is_err}")
print(text)
ts = []
for _ in range(20):
    t, _, _, e = m.call("browser_state", {})
    ts.append(t)
ts.sort()
print(f"20 more calls: p50={ts[10]:.1f} ms max={ts[-1]:.1f} ms")
ok = (not is_err) and "FakeBrave" in text and '[0] Button "Go"' in text and set(seen[:2]) == {"tabs.list", "page.state"}
m.close()
host.stdin.close()
time.sleep(0.5)
hosts_dir = os.path.join(os.environ["LOCALAPPDATA"], "fastuse", "hosts")
gone = [f for f in os.listdir(hosts_dir) if f == f"{host.pid}.json"] if os.path.isdir(hosts_dir) else []
host.wait(timeout=5)
print("host exit code", host.returncode, "| leftover host files:", gone)
print("PASS" if ok and host.returncode == 0 and not gone else "FAIL")
