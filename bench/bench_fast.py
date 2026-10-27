import os
import re, json, subprocess, statistics as st, time

EXE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target", "release", "fastuse.exe")
p = subprocess.Popen([EXE], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=0)
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
    return (time.perf_counter() - t) * 1000, r


def text(r):
    return "\n".join(c.get("text", "") for c in r["result"]["content"] if c["type"] == "text")


def bench(name, args, tool, n_=20, warm=3):
    for _ in range(warm):
        call(tool, args)
    xs = []
    for _ in range(n_):
        ms, r = call(tool, args)
        xs.append(ms)
    xs.sort()
    print(f"{name:36s} p50={st.median(xs):7.1f}ms p95={xs[min(len(xs)-1,int(len(xs)*.95))]:7.1f}ms min={xs[0]:7.1f}ms", flush=True)
    return r


rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "b", "version": "0"}})
print("tools:", [t["name"] for t in rpc("tools/list", {})["result"]["tools"]])
ms, r = call("list_windows")
l = [x for x in text(r).splitlines() if "BenchTarget" in x][0]
W = dict(window_id=int(re.search(r"window_id: (\d+)", l).group(1)))

bench("list_windows", {}, "list_windows")
bench("list_windows exe=true", {"exe": True}, "list_windows")
r = bench("get_window_state (text only)", W, "get_window_state")
print(text(r))
bench("get_window_state +screenshot", dict(W, screenshot=True), "get_window_state")
r = bench("screenshot print full", W, "screenshot")
print(text(r))
r = bench("screenshot screen(BitBlt) full", dict(W, mode="screen"), "screenshot")
print(text(r))
bench("screenshot print max_dim=512", dict(W, max_dim=512), "screenshot")
r = bench("screenshot region 200x100", dict(W, region=[0, 0, 200, 100]), "screenshot")
print(text(r))
r = bench("screenshot cached region (50ms)", dict(W, region=[0, 0, 200, 100], max_age_ms=60000), "screenshot")
print(text(r))
p.terminate()
