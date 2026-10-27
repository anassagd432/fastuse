import json, re, subprocess, sys, time, threading, os, statistics as st

B = os.path.dirname(os.path.abspath(__file__))
EXE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target", "release", "fastuse.exe")
PROMPT = "Using ONLY the fastuse MCP tools: in the window titled BenchTarget, set the text field to hello-from-agy and click the Go button. Reply with the text field's final value."
N = int(sys.argv[1]) if len(sys.argv) > 1 else 6


def start_target():
    return subprocess.Popen(["powershell", "-NoProfile", "-STA", "-File", os.path.join(B, "target.ps1")])


def field_value():
    p = subprocess.Popen([EXE], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    inp = ('{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"v","version":"0"}}}\n'
           '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_window_state","arguments":{"window":"BenchTarget"}}}\n').encode()
    out, _ = p.communicate(inp, timeout=20)
    last = out.decode().strip().splitlines()[-1]
    txt = json.loads(last)["result"]["content"][0]["text"]
    m = re.search(r'Edit "input" value="([^"]*)"', txt)
    return m.group(1) if m else ""


t_spawn = time.time()
p = subprocess.Popen(["agy", "--model", "gemini-3.8-flash-low", "--print-timeout", "150s",
                      "--input-format", "stream-json", "--output-format", "stream-json"],
                     stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=0)
events = []
threading.Thread(target=lambda: [events.append(l) for l in p.stdout], daemon=True).start()
while not any(b'"event":"init"' in l for l in events):
    time.sleep(0.05)
print(f"agy startup to init: {time.time()-t_spawn:.1f}s", flush=True)

rows = []
for i in range(1, N + 1):
    tgt = start_target()
    time.sleep(3)
    mark = len(events)
    t0 = time.time()
    p.stdin.write((json.dumps({"event": "user", "message": {"content": PROMPT}}) + "\n").encode())
    p.stdin.flush()
    res = None
    seq = []
    while time.time() - t0 < 170 and not res:
        time.sleep(0.05)
        for l in events[mark:]:
            try:
                e = json.loads(l)
            except Exception:
                continue
            if e.get("event") == "result":
                res = e["result"]
    wall = time.time() - t0
    for l in events[mark:]:
        try:
            e = json.loads(l)
        except Exception:
            continue
        if e.get("event") == "step_update":
            su = e["step_update"]
            if su.get("state") == "DONE" and su.get("step_type") == "tool":
                ti = su.get("tool_info", {})
                seq.append(ti.get("parameters", {}).get("ToolName") or "(" + su.get("tool_name", "?") + ")")
    try:
        val = field_value()
    except Exception as ex:
        val = f"check failed: {ex}"
    ok = val == "clicks=1"
    rows.append((wall, len(seq), ok))
    print(f"task {i}: wall={wall:.1f}s steps={len(seq)} ok={ok} value={val!r} seq={' > '.join(seq)}", flush=True)
    tgt.kill()
    time.sleep(1)

p.kill()
w = [r[0] for r in rows if r[2]]
print(f"success {sum(r[2] for r in rows)}/{len(rows)}; successful wall median={st.median(w) if w else float('nan'):.1f}s min={min(w) if w else float('nan'):.1f}s")
