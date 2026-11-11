import os
import requests

env_file = r"C:\Users\Anass\AppData\Local\hermes\.env"
api_key = None
with open(env_file, "r", encoding="utf-8") as f:
    for line in f:
        if line.startswith("ELEVENLABS_API_KEY="):
            api_key = line.split("=", 1)[1].strip()
            break

if not api_key:
    raise ValueError("No ELEVENLABS_API_KEY found")

# Voice: Charlie - Deep, Confident, Energetic (IKne3meq5aSn9XLyUdCD)
# Or Anass Agdi (E0CbRySwbPdp2He8A4Vk)
VOICE_ID = "IKne3meq5aSn9XLyUdCD"

script = """Every AI computer use tool today has the exact same flaw: it hijacks your real mouse, freezes your screen, and burns forty seconds per action.

Not anymore. Meet fastuse.

Re-engineered from the metal up in native Rust, fastuse gives your AI agents sub-millisecond vision and 1.8 millisecond background clicks. Your mouse cursor stays completely free. You keep typing and coding, while the agent operates windows in the background.

And inside your browser? fastuse drives your actual Brave, Chrome, and Edge tabs with all your logins and cookies intact. No headless bot logouts.

Best of all: it installs into Google Antigravity with a single prompt.

Stop waiting forty seconds. Take back your cursor. Fastuse."""

url = f"https://api.elevenlabs.io/v1/text-to-speech/{VOICE_ID}"

headers = {
    "Accept": "audio/mpeg",
    "Content-Type": "application/json",
    "xi-api-key": api_key
}

data = {
    "text": script,
    "model_id": "eleven_multilingual_v2",
    "voice_settings": {
        "stability": 0.40,
        "similarity_boost": 0.85,
        "style": 0.55,
        "use_speaker_boost": True
    }
}

print(f"Generating voiceover with voice {VOICE_ID}...")
response = requests.post(url, json=data, headers=headers)

if response.status_code == 200:
    out_path = r"demo-video\public\audio\voiceover.mp3"
    with open(out_path, "wb") as f:
        f.write(response.content)
    print(f"Voiceover saved successfully to {out_path} ({len(response.content)} bytes)")
else:
    print(f"Error {response.status_code}: {response.text}")
