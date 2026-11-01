"""Captures the test window area at a few moments after an act call, to eyeball the agent cursor."""
import base64, json, os, re, subprocess, time

B = os.path.dirname(os.path.abspath(__file__))
EXE = os.path.join(B, "..", "target", "release", "fastuse.exe")
tgt = subprocess.Popen(["powershell", "-NoProfile", "-STA", "-File", os.path.join(B, "target.ps1")])
time.sleep(3)
p = subprocess.Popen([EXE], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=0)
n = 0


def call(name, args):
    global n
    n += 1
    p.stdin.write((json.dumps({"jsonrpc": "2.0", "id": n, "method": "tools/call", "params": {"name": name, "arguments": args}}) + "\n").encode())
    p.stdin.flush()
    return json.loads(p.stdout.readline())["result"]


p.stdin.write((json.dumps({"jsonrpc": "2.0", "id": 99, "method": "initialize", "params": {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}}) + "\n").encode())
p.stdin.flush()
p.stdout.readline()

line = [l for l in call("list_windows", {})["content"][0]["text"].splitlines() if "BenchTarget" in l][0]
wl, wt = map(int, re.search(r"@\((-?\d+),(-?\d+)\)", line).groups())
ww, wh = map(int, re.search(r"\] (\d+)x(\d+)", line).groups())


def shot(tag):
    r = call("screenshot", {"screen": True, "region": [max(wl, 0), max(wt, 0), ww, wh], "max_dim": 0, "quality": 90})
    img = [c for c in r["content"] if c["type"] == "image"][0]
    open(os.path.join(B, f"look_{tag}.jpg"), "wb").write(base64.b64decode(img["data"]))


t0 = time.time()
call("act", {"window": "BenchTarget", "steps": [{"tool": "set_value", "target": "input", "value": "x"}, {"tool": "click", "target": "Go"}], "observe": False})
print("act returned in", round((time.time() - t0) * 1000), "ms")
for tag, until in (("a_glide", 0.12), ("b_ripple", 0.42), ("c_rest", 1.2)):
    time.sleep(max(0, until - (time.time() - t0)))
    shot(tag)
time.sleep(9)
shot("d_after_idle")
p.kill()
tgt.kill()
