"""Triggers the agent overlay (cursor, glow, banner) and saves desktop screenshots so you can eyeball it.
Usage: python bench/overlay_look.py [path-to-fastuse.exe]   (sets FASTUSE_RECORDABLE=1 so the glow is captured)"""
import base64, json, os, subprocess, sys, time

B = os.path.dirname(os.path.abspath(__file__))
EXE = sys.argv[1] if len(sys.argv) > 1 else os.path.join(B, "..", "target", "release", "fastuse.exe")
env = dict(os.environ, FASTUSE_RECORDABLE="1")
tgt = subprocess.Popen(["powershell", "-NoProfile", "-STA", "-File", os.path.join(B, "target.ps1")])
time.sleep(3)
p = subprocess.Popen([EXE], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=0, env=env)
n = 0


def rpc(method, params):
    global n
    n += 1
    p.stdin.write((json.dumps({"jsonrpc": "2.0", "id": n, "method": method, "params": params}) + "\n").encode())
    p.stdin.flush()
    return json.loads(p.stdout.readline())["result"]


rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}})
t0 = time.time()
rpc("tools/call", {"name": "act", "arguments": {"window": "BenchTarget", "steps": [{"tool": "click", "target": "Go"}], "observe": False}})
print("act returned in", round((time.time() - t0) * 1000), "ms")
for tag, until in (("a_start", 0.15), ("b_arrived", 0.9), ("c_steady", 2.5)):
    time.sleep(max(0, until - (time.time() - t0)))
    r = rpc("tools/call", {"name": "screenshot", "arguments": {"screen": True, "max_dim": 1400, "quality": 85}})
    img = [c for c in r["content"] if c["type"] == "image"][0]
    open(os.path.join(B, f"overlay_{tag}.jpg"), "wb").write(base64.b64decode(img["data"]))
    print("saved", tag)
p.kill()
tgt.kill()
