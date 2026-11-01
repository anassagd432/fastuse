import ctypes, json, subprocess, threading, time, os
from ctypes import wintypes

B = os.path.dirname(os.path.abspath(__file__))
u = ctypes.windll.user32
u.FindWindowW.restype = wintypes.HWND
u.FindWindowW.argtypes = [wintypes.LPCWSTR, wintypes.LPCWSTR]
u.IsWindowVisible.argtypes = [wintypes.HWND]

tgt = subprocess.Popen(["powershell", "-NoProfile", "-STA", "-File", os.path.join(B, "target.ps1")])
time.sleep(3)
p = subprocess.Popen(["agy", "--model", "gemini-3.8-flash-low", "--print-timeout", "120s",
                      "--input-format", "stream-json", "--output-format", "stream-json"],
                     stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=0)
events = []
threading.Thread(target=lambda: [events.append(l) for l in p.stdout], daemon=True).start()
while not any(b'"event":"init"' in l for l in events):
    time.sleep(0.05)

prompt = "Using ONLY the fastuse MCP tools: in the window titled BenchTarget, set the text field to hello-from-agy and click the Go button. Reply with the text field's final value."
t0 = time.time()
p.stdin.write((json.dumps({"event": "user", "message": {"content": prompt}}) + "\n").encode())
p.stdin.flush()

seen_window = False
visible_samples = 0
total = 0
tools = []
done = False
while time.time() - t0 < 110 and not done:
    h = u.FindWindowW("FastuseAgentCursor", None)
    total += 1
    if h:
        seen_window = True
        if u.IsWindowVisible(h):
            visible_samples += 1
    for l in events:
        if b'"event":"result"' in l:
            done = True
    time.sleep(0.05)

for l in events:
    try:
        e = json.loads(l)
    except Exception:
        continue
    if e.get("event") == "step_update":
        su = e["step_update"]
        if su.get("state") == "DONE" and su.get("step_type") == "tool":
            ti = su.get("tool_info", {})
            tools.append(ti.get("parameters", {}).get("ToolName") or "(" + su.get("tool_name", "?") + ")")
print("tools used:", " > ".join(tools))
print(f"overlay window ever existed: {seen_window}; visible in {visible_samples} of {total} samples ({visible_samples*0.05:.1f}s visible)")
p.kill()
tgt.kill()
