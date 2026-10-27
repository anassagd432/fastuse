import os
import re, json, statistics as st, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mcpclient import MCP, text

m = MCP()
t = text(m.call("list_windows")[1])
l = [x for x in t.splitlines() if "BenchTarget" in x][0]
W = dict(pid=int(re.search(r"pid (\d+)", l).group(1)), window_id=int(re.search(r"window_id: (\d+)", l).group(1)))


def snap():
    ms, r = m.call("get_window_state", W)
    s = text(r)
    return ms, s, re.search(r"snapshot_id=(\w+)", s).group(1)


def clicks(s):
    mm = re.search(r'Edit "input" \[value="clicks=(\d+)"', s)
    return int(mm.group(1)) if mm else 0


def stats(name, xs):
    xs = sorted(xs)
    print(f"{name:40s} n={len(xs):3d} p50={st.median(xs):7.1f}ms p95={xs[min(len(xs)-1,int(len(xs)*.95))]:7.1f}ms min={xs[0]:7.1f}ms", flush=True)


# 1. state read
_, s0, sid = snap()
base = clicks(s0)

# 2. click by token, verified
xs = []
for i in range(20):
    ms, r = m.call("click", dict(W, element_token=f"{sid}:1"))
    if "error" in text(r).lower() or r.get("result", {}).get("isError"):
        print("click err:", text(r)[:200]); break
    xs.append(ms)
stats("click(token) [same snapshot]", xs)
_, s1, sid = snap()
print("verified click count:", clicks(s1) - base, "of", len(xs))

# 3. read + click as two calls
xs = []
for i in range(10):
    ms1, _, sid = snap()
    ms2, r = m.call("click", dict(W, element_token=f"{sid}:1"))
    xs.append(ms1 + ms2)
stats("state + click (2 calls)", xs)

# 4. run_steps click+observe (one call)
xs = []
for i in range(10):
    ms, r = m.call("run_steps", dict(W, steps=[{"tool": "click", "args": {"element_token": f"{sid}:1"}}], observe={}))
    xs.append(ms)
    if i == 0:
        print("run_steps resp:", text(r)[:300].replace("\n", " | "))
    ms1, _, sid = snap()
stats("run_steps click+observe", xs)
_, s2, sid = snap()
print("verified total clicks:", clicks(s2) - base)

# 5. diff read
_, _, sid = snap()
xs = []
for i in range(15):
    ms, r = m.call("get_window_state", dict(W, since=sid))
    xs.append(ms)
    sid = re.search(r"snapshot_id=(\w+)", text(r)).group(1)
stats("get_window_state since= (diff)", xs)
print(text(r)[:200].replace("\n", " | "))

# 6. text set via set_value
xs = []
for i in range(10):
    ms1, _, sid = snap()
    ms, r = m.call("set_value", dict(W, element_token=f"{sid}:0", value="x" * 20))
    xs.append(ms)
stats("set_value 20ch", xs)

# 7. get_window_state with screenshot size?
ms, r = m.call("get_window_state", W)
res = r.get("result", {})
print("content types:", [(c.get("type"), len(c.get("data", "")) if c.get("type") == "image" else len(c.get("text", ""))) for c in res.get("content", [])])
m.close()
