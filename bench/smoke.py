"""Cross-platform smoke test: starts fastuse over stdio, lists windows, reads one window's tree,
and times a few repeated reads. Usage:  python3 bench/smoke.py "<part of a window title>"
Prints only window titles/sizes and the first lines of the tree of the window you name."""
import json, os, subprocess, sys, time, statistics as st

here = os.path.dirname(os.path.abspath(__file__))
exe = os.path.join(here, "..", "target", "release", "fastuse" + (".exe" if os.name == "nt" else ""))
title = sys.argv[1] if len(sys.argv) > 1 else ""

p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
n = 0


def rpc(method, params):
    global n
    n += 1
    p.stdin.write((json.dumps({"jsonrpc": "2.0", "id": n, "method": method, "params": params}) + "\n").encode())
    p.stdin.flush()
    while True:
        r = json.loads(p.stdout.readline())
        if r.get("id") == n:
            return r


def call(name, args=None):
    t = time.perf_counter()
    r = rpc("tools/call", {"name": name, "arguments": args or {}})
    ms = (time.perf_counter() - t) * 1000
    res = r["result"]
    return ms, "\n".join(c.get("text", "") for c in res["content"] if c["type"] == "text"), res.get("isError", False)


init = rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "smoke", "version": "0"}})
print("server:", init["result"]["serverInfo"])
ms, t, err = call("list_windows")
print(f"\nlist_windows: {ms:.1f} ms{' (ERROR)' if err else ''}")
for line in t.splitlines()[:15]:
    print("  ", line)
if title:
    ms, t, err = call("get_window_state", {"window": title, "max_elements": 60})
    print(f"\nget_window_state('{title}'): {ms:.1f} ms{' (ERROR)' if err else ''}")
    for line in t.splitlines()[:45]:
        print("  ", line)
    if not err:
        xs = sorted(call("get_window_state", {"window": title})[0] for _ in range(10))
        print(f"\n10 repeated reads: p50={st.median(xs):.1f} ms  min={xs[0]:.1f} ms  max={xs[-1]:.1f} ms")
p.kill()
err_text = p.stderr.read().decode(errors="ignore").strip()
if err_text:
    print("\nstderr:", err_text[:500])
