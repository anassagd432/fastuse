"""
Bespoke Cyberpunk / Tech Synthwave Soundtrack Generator for Fastuse Motion Graphics Video.
Generates an energetic, 125 BPM electronic backing track with:
- Heavy 808 sub kick with punchy transient click
- Snappy crisp claps and rolling hi-hats
- Deep reese bassline with filter modulation
- Cyber arpeggiated synth line (D minor pentatonic)
- Riser sweeps and cinematic impact booms at scene transitions
- Sidechain ducking to give that modern pumping motion graphics feel
"""

import math
import numpy as np
import wave
import subprocess

SAMPLE_RATE = 44100
DURATION = 49.5
TOTAL_SAMPLES = int(SAMPLE_RATE * DURATION)
BPM = 125.0
BEAT_SEC = 60.0 / BPM
BAR_SEC = BEAT_SEC * 4.0

t = np.linspace(0, DURATION, TOTAL_SAMPLES, endpoint=False)

def create_kick(t_rel, decay=0.25):
    """Punchy 808 pitch-dropped kick drum with click."""
    freq = 150.0 * np.exp(-t_rel / 0.04) + 45.0
    phase = 2.0 * np.pi * np.cumsum(freq) / SAMPLE_RATE
    body = np.sin(phase) * np.exp(-t_rel / decay)
    # transient click
    click = np.random.uniform(-1, 1, len(t_rel)) * np.exp(-t_rel / 0.005) * 0.4
    return np.clip(body + click, -1.0, 1.0)

def create_snare(t_rel):
    """Crisp modern clap / snare with filtered noise."""
    noise = np.random.uniform(-1, 1, len(t_rel)) * np.exp(-t_rel / 0.12)
    tone = np.sin(2.0 * np.pi * 185.0 * t_rel) * np.exp(-t_rel / 0.08) * 0.5
    return noise * 0.7 + tone * 0.3

def create_hihat(t_rel, closed=True):
    """Crisp metallic hi-hat."""
    decay = 0.04 if closed else 0.18
    noise = np.random.uniform(-1, 1, len(t_rel)) * np.exp(-t_rel / decay)
    return noise

# 1. BUILD DRUM STEM
drum_left = np.zeros(TOTAL_SAMPLES)
drum_right = np.zeros(TOTAL_SAMPLES)

# Beat times
kick_samples = int(SAMPLE_RATE * 0.35)
snare_samples = int(SAMPLE_RATE * 0.25)
hat_samples = int(SAMPLE_RATE * 0.1)

t_kick = np.linspace(0, 0.35, kick_samples, endpoint=False)
t_snare = np.linspace(0, 0.25, snare_samples, endpoint=False)
t_hat = np.linspace(0, 0.1, hat_samples, endpoint=False)

kick_hit = create_kick(t_kick)
snare_hit = create_snare(t_snare)
hat_hit = create_hihat(t_hat)

total_beats = int(DURATION / BEAT_SEC)

for b in range(total_beats):
    beat_time = b * BEAT_SEC
    idx = int(beat_time * SAMPLE_RATE)
    
    # Intro is ambient (0 to 6s) - kick starts at 6s (bar 3)
    if beat_time < 5.8:
        continue
    
    # Kick on beats 0, 1, 2, 3 (four on the floor driving energy!)
    if idx + kick_samples <= TOTAL_SAMPLES:
        drum_left[idx:idx+kick_samples] += kick_hit * 0.75
        drum_right[idx:idx+kick_samples] += kick_hit * 0.75
        
    # Snare on beats 2 and 4 (beats index 1 and 3 in zero-indexed bar)
    bar_beat = b % 4
    if bar_beat in (1, 3):
        if idx + snare_samples <= TOTAL_SAMPLES:
            drum_left[idx:idx+snare_samples] += snare_hit * 0.55
            drum_right[idx:idx+snare_samples] += snare_hit * 0.55
            
    # Hi-hats on off-beats (8th notes and 16th notes)
    off_idx = int((beat_time + BEAT_SEC * 0.5) * SAMPLE_RATE)
    if off_idx + hat_samples <= TOTAL_SAMPLES:
        drum_left[off_idx:off_idx+hat_samples] += hat_hit * 0.35
        drum_right[off_idx:off_idx+hat_samples] += hat_hit * 0.30
        
    # 16th note hats in intense sections (22s to 49s)
    if beat_time >= 22.0:
        for sixteenth in [0.25, 0.75]:
            six_idx = int((beat_time + BEAT_SEC * sixteenth) * SAMPLE_RATE)
            if six_idx + hat_samples <= TOTAL_SAMPLES:
                pan = 0.2 if sixteenth == 0.25 else 0.4
                drum_left[six_idx:six_idx+hat_samples] += hat_hit * (0.25 * (1 - pan))
                drum_right[six_idx:six_idx+hat_samples] += hat_hit * (0.25 * pan)

# 2. BUILD BASS STEM (Deep rolling D minor synth bass)
# Notes: D1=36.7Hz, F1=43.65Hz, G1=48.99Hz, A1=55.0Hz, C2=65.4Hz
bass_left = np.zeros(TOTAL_SAMPLES)
bass_right = np.zeros(TOTAL_SAMPLES)

root_freqs = [36.7, 36.7, 43.65, 48.99, 36.7, 36.7, 55.0, 48.99]

for bar in range(int(DURATION / BAR_SEC) + 1):
    bar_start = bar * BAR_SEC
    freq = root_freqs[bar % len(root_freqs)]
    
    # 8th note rolling bass pulses
    for eighth in range(8):
        pulse_start = bar_start + eighth * (BEAT_SEC * 0.5)
        if pulse_start >= DURATION:
            break
        pulse_idx = int(pulse_start * SAMPLE_RATE)
        pulse_len = int(BEAT_SEC * 0.45 * SAMPLE_RATE)
        if pulse_idx + pulse_len > TOTAL_SAMPLES:
            continue
            
        t_pulse = np.linspace(0, BEAT_SEC * 0.45, pulse_len, endpoint=False)
        # Rich sawtooth wave with odd harmonics
        saw = (
            np.sin(2.0 * np.pi * freq * t_pulse) * 0.6 +
            np.sin(2.0 * np.pi * freq * 2 * t_pulse) * 0.3 +
            np.sin(2.0 * np.pi * freq * 3 * t_pulse) * 0.15
        )
        env = np.exp(-t_pulse / 0.18)
        bass_hit = np.tanh(saw * 2.0) * env * 0.45
        
        # Bass volume starts softly at intro, peaks during main drops
        vol = 0.25 if pulse_start < 6.0 else (0.55 if pulse_start < 40.0 else 0.65)
        bass_left[pulse_idx:pulse_idx+pulse_len] += bass_hit * vol
        bass_right[pulse_idx:pulse_idx+pulse_len] += bass_hit * vol

# 3. BUILD CYBER ARPEGGIO STEM
# Fast 16th-note plucks in D minor pentatonic: D3, F3, G3, A3, C4, D4
arp_notes = [146.83, 174.61, 196.00, 220.00, 261.63, 293.66, 349.23, 293.66]
arp_left = np.zeros(TOTAL_SAMPLES)
arp_right = np.zeros(TOTAL_SAMPLES)

sixteenth_sec = BEAT_SEC / 4.0
total_sixteenths = int(DURATION / sixteenth_sec)

for s in range(total_sixteenths):
    time_s = s * sixteenth_sec
    idx = int(time_s * SAMPLE_RATE)
    plen = int(sixteenth_sec * 0.9 * SAMPLE_RATE)
    if idx + plen > TOTAL_SAMPLES:
        break
        
    freq = arp_notes[s % len(arp_notes)]
    # Octave up every 16 steps
    if (s // 16) % 2 == 1:
        freq *= 2.0
        
    t_p = np.linspace(0, sixteenth_sec * 0.9, plen, endpoint=False)
    # Bright FM synth pluck
    mod = np.sin(2.0 * np.pi * (freq * 2.0) * t_p) * 2.0 * np.exp(-t_p / 0.05)
    carrier = np.sin(2.0 * np.pi * freq * t_p + mod) * np.exp(-t_p / 0.08)
    
    # Stereo ping-pong pan
    pan = 0.5 + 0.4 * np.sin(s * 0.7)
    vol = 0.18 if time_s < 6.0 else (0.28 if time_s < 40.0 else 0.35)
    arp_left[idx:idx+plen] += carrier * vol * (1.0 - pan)
    arp_right[idx:idx+plen] += carrier * vol * pan

# 4. TRANSITION RISERS & IMPACT BOOMS
sfx_left = np.zeros(TOTAL_SAMPLES)
sfx_right = np.zeros(TOTAL_SAMPLES)

transition_times = [0.0, 6.0, 13.0, 22.0, 32.0, 40.0]

for tt in transition_times:
    # Impact sub boom
    boom_len = int(SAMPLE_RATE * 1.5)
    b_idx = int(tt * SAMPLE_RATE)
    if b_idx + boom_len <= TOTAL_SAMPLES:
        t_b = np.linspace(0, 1.5, boom_len, endpoint=False)
        boom = np.sin(2.0 * np.pi * (70.0 * np.exp(-t_b / 0.4) + 30.0) * t_b) * np.exp(-t_b / 0.5) * 0.6
        sfx_left[b_idx:b_idx+boom_len] += boom
        sfx_right[b_idx:b_idx+boom_len] += boom

    # Riser sweep leading into the drop
    if tt >= 6.0:
        riser_len = int(SAMPLE_RATE * 1.5)
        r_start = int((tt - 1.5) * SAMPLE_RATE)
        if r_start >= 0 and r_start + riser_len <= TOTAL_SAMPLES:
            t_r = np.linspace(0, 1.5, riser_len, endpoint=False)
            sweep_freq = 200.0 + 1200.0 * (t_r / 1.5)**2
            phase = 2.0 * np.pi * np.cumsum(sweep_freq) / SAMPLE_RATE
            noise = np.random.uniform(-1, 1, riser_len) * (t_r / 1.5)**2 * 0.3
            riser = (np.sin(phase) * 0.3 + noise) * (t_r / 1.5) * 0.4
            sfx_left[r_start:r_start+riser_len] += riser
            sfx_right[r_start:r_start+riser_len] += riser

# MIX TOGETHER
master_left = drum_left + bass_left + arp_left + sfx_left
master_right = drum_right + bass_right + arp_right + sfx_right

# Master limiter & gentle fadeout at end
fade_samples = int(SAMPLE_RATE * 1.5)
fade_env = np.ones(TOTAL_SAMPLES)
fade_env[-fade_samples:] = np.linspace(1.0, 0.0, fade_samples)

master_left *= fade_env
master_right *= fade_env

# Soft saturation ceiling
master_left = np.tanh(master_left * 1.2) * 0.85
master_right = np.tanh(master_right * 1.2) * 0.85

# Convert to 16-bit PCM
pcm_left = (master_left * 32767).astype(np.int16)
pcm_right = (master_right * 32767).astype(np.int16)
stereo_pcm = np.empty((TOTAL_SAMPLES * 2,), dtype=np.int16)
stereo_pcm[0::2] = pcm_left
stereo_pcm[1::2] = pcm_right

wav_path = "c:/Users/Anass/Documents/GitHub/fastuse/demo-video/public/audio/cyber_beat.wav"
with wave.open(wav_path, "wb") as wf:
    wf.setnchannels(2)
    wf.setsampwidth(2)
    wf.setframerate(SAMPLE_RATE)
    wf.writeframes(stereo_pcm.tobytes())

print(f"Generated soundtrack WAV: {wav_path}")

# Now mix with voiceover into final soundtrack MP3 using ffmpeg
# Voiceover is given +2dB boost and slight sidechain compression on the beat
mix_cmd = [
    "ffmpeg", "-y",
    "-i", "c:/Users/Anass/Documents/GitHub/fastuse/demo-video/public/audio/cyber_beat.wav",
    "-i", "c:/Users/Anass/Documents/GitHub/fastuse/demo-video/public/audio/voiceover.mp3",
    "-filter_complex",
    "[0:a]volume=0.45[music];[1:a]volume=1.4[voice];[music][voice]amix=inputs=2:duration=first:dropout_transition=2[out]",
    "-map", "[out]",
    "-b:a", "256k",
    "c:/Users/Anass/Documents/GitHub/fastuse/demo-video/public/audio/master_audio.mp3"
]

subprocess.run(mix_cmd, check=True)
print("Master mixed audio created: master_audio.mp3")
