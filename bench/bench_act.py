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
    ms = (time.perf_counter() - t) * 1000
    res = r["result"]
    txt = "\n".join(c.get("text", "") for c in res["content"] if c["type"] == "text")
    return ms, txt, res.get("isError", False)


rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "b", "version": "0"}})
_, t, _ = call("list_windows")
l = [x for x in t.splitlines() if "BenchTarget" in x][0]
wid = int(re.search(r"window_id: (\d+)", l).group(1))
wl, wt = map(int, re.search(r"@\((-?\d+),(-?\d+)\)", l).groups())
W = dict(window_id=wid)


def state():
    ms, t, _ = call("get_window_state", W)
    sid = re.search(r"snapshot_id=(\w+)", t).group(1)
    return ms, t, sid


def val(t):
    m = re.search(r'Edit "input" value="([^"]*)"', t)
    return m.group(1) if m else ""


def tok_of(t, sid, label):
    for ln in t.splitlines():
        if label in ln:
            m = re.search(r"\[(\d+)\]", ln)
            if m:
                return f"{sid}:{m.group(1)}"


def rep(name, xs, extra=""):
    xs = sorted(xs)
    print(f"{name:38s} p50={st.median(xs):7.1f}ms p95={xs[min(len(xs)-1,int(len(xs)*.95))]:7.1f}ms min={xs[0]:7.1f}ms {extra}", flush=True)


_, t, sid = state()
go = tok_of(t, sid, 'Button "Go"')
inp = tok_of(t, sid, 'Edit "input"')
print("tokens", go, inp)

# 1 click token
xs = []
for _ in range(20):
    ms, o, e = call("click", dict(element_token=go))
    assert not e, o
    xs.append(ms)
_, t, sid = state()
rep("click(token) x20", xs, f"-> input shows '{val(t)}' (expect clicks=20)")

# 2 two-call vs run_steps
xs = []
for _ in range(15):
    ms1, t, sid = state()
    g = tok_of(t, sid, 'Button "Go"')
    ms2, o, e = call("click", dict(element_token=g))
    xs.append(ms1 + ms2)
rep("state + click (2 calls)", xs)
_, t, sid = state()
xs = []
for _ in range(15):
    g = tok_of(t, sid, 'Button "Go"')
    ms, o, e = call("run_steps", dict(window_id=wid, steps=[{"tool": "click", "args": {"element_token": g}}], observe={}))
    assert not e, o
    sid = re.search(r"snapshot_id=(\w+)", o).group(1)
    t = o
    xs.append(ms)
rep("run_steps click+observe (1 call)", xs, f"-> '{val(t)}' (expect clicks=50)")

# 3 set_value
_, t, sid = state()
xs = []
for _ in range(15):
    ms, o, e = call("set_value", dict(element_token=tok_of(t, sid, 'Edit "input"'), value="x" * 20))
    assert not e, o
    xs.append(ms)
_, t, sid = state()
rep("set_value 20ch", xs, f"-> len={len(val(t))}")

# 4 type_text background
inp = tok_of(t, sid, 'Edit "input"')
call("set_value", dict(element_token=inp, value=""))
xs = []
for _ in range(10):
    call("set_value", dict(element_token=inp, value=""))
    ms, o, e = call("type_text", dict(element_token=inp, text="hello benchmark 1234"))
    assert not e, o
    xs.append(ms)
time.sleep(0.05)
_, t, sid = state()
rep("type_text 20ch (bg WM_CHAR)", xs, f"-> '{val(t)}'")
big = "The quick brown fox jumps over the lazy dog. " * 22
call("set_value", dict(element_token=inp, value=""))
ms, o, e = call("type_text", dict(element_token=inp, text=big))
time.sleep(0.2)
_, t, sid = state()
print(f"type_text {len(big)}ch: call took {ms:.1f}ms, field now {len(val(t))} chars")

# 5 coordinate click background on Go button
_, t, sid = state()
ln = [x for x in t.splitlines() if 'Button "Go"' in x][0]
bx, by, bw, bh = map(int, re.search(r"@\((-?\d+),(-?\d+),(\d+),(\d+)\)", ln).groups())
cx, cy = bx + bw // 2 - wl, by + bh // 2 - wt
xs = []
for _ in range(10):
    ms, o, e = call("click", dict(window_id=wid, x=cx, y=cy))
    assert not e, o
    xs.append(ms)
time.sleep(0.1)
_, t, sid = state()
rep("click(x,y) background PostMessage", xs, f"-> '{val(t)}' (expect clicks=... increased)")

# 6 press_key
ms, o, e = call("press_key", dict(window_id=wid, key="enter"))
print("press_key enter:", round(ms, 1), "ms", o, "ERR" if e else "")

# 7 hotkey + foreground typing (visible focus change, own window only)
call("set_value", dict(element_token=tok_of(t, sid, 'Edit "input"'), value="select me"))
ms1, o1, e1 = call("hotkey", dict(window_id=wid, keys="ctrl+a"))
print("hotkey:", round(ms1, 1), "ms", o1, "ERR" if e1 else "")
if not e1:
    ms2, o2, e2 = call("type_text", dict(window_id=wid, text="Z", delivery_mode="foreground"))
    print("fg type:", round(ms2, 1), "ms", o2, "ERR" if e2 else "")
    time.sleep(0.1)
    _, t, sid = state()
    print("after ctrl+a then 'Z':", repr(val(t)), "(expect 'Z' if the focused control was the textbox)")

# 8 stale token guard
ms, o, e = call("click", dict(element_token="s00000001:0"))
print("stale token ->", "ERR" if e else "OK?", o[:90])
p.terminate()
