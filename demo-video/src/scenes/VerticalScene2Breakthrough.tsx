import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { TunnelWarp } from "../components/TunnelWarp";
import { StarburstBadge } from "../components/StarburstBadge";
import { CinematicHud } from "../components/CinematicHud";
import { KineticWords } from "../components/KineticWords";
import { THEME } from "../theme";

const BENCHMARKS = [
  {
    label: "Window Enumeration",
    legacy: "437 ms",
    fast: "0.9 ms",
    speedup: "485x FASTER",
  },
  {
    label: "Background Click",
    legacy: "16.0 ms",
    fast: "1.8 ms",
    speedup: "9x FASTER",
  },
  {
    label: "UI Tree Cache & Walk",
    legacy: "160 ms",
    fast: "29 ms",
    speedup: "5.5x FASTER",
  },
];

export const VerticalScene2Breakthrough: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoSpring = spring({
    frame,
    fps,
    config: { damping: 12, stiffness: 140 },
  });

  return (
    <div
      style={{
        width: 1080,
        height: 1920,
        backgroundColor: "#030712",
        position: "relative",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "80px 40px",
      }}
    >
      {/* Generative 3D Tunnel Warp Background */}
      <TunnelWarp speed={1.4} glowColor="#00F0FF" />

      {/* Cinematic HUD Overlay */}
      <CinematicHud
        sceneIndex="02 / 06"
        sceneTitle="KINETIC BREAKTHROUGH: THE RUST ENGINE"
        fpsText="144.0 FPS (NATIVE)"
        statusText="ZERO LATENCY PIPELINE ACTIVE"
      />

      {/* Floating Starburst Badge */}
      <div
        style={{
          position: "absolute",
          top: 140,
          right: 60,
          zIndex: 30,
        }}
      >
        <StarburstBadge
          text="PURE RUST"
          subtext="144 FPS"
          color="#00F0FF"
          textColor="#030712"
          size={190}
          rotationOffset={-10}
        />
      </div>

      {/* Center 3D Benchmark Cockpit Card */}
      <div
        style={{
          zIndex: 20,
          width: "100%",
          maxWidth: 920,
          marginTop: 220,
          display: "flex",
          flexDirection: "column",
          gap: 20,
        }}
      >
        {/* Brand Tag Pill */}
        <div
          style={{
            alignSelf: "center",
            display: "flex",
            alignItems: "center",
            gap: 14,
            background: "rgba(0, 240, 255, 0.15)",
            border: "2px solid #00F0FF",
            boxShadow: "0 0 35px rgba(0, 240, 255, 0.45)",
            padding: "12px 36px",
            borderRadius: 999,
            transform: `scale(${logoSpring})`,
          }}
        >
          <span style={{ fontSize: 24 }}>⚡</span>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 20,
              fontWeight: 800,
              color: "#00F0FF",
              letterSpacing: "0.15em",
            }}
          >
            FASTUSE RUST CORE // v0.2.0
          </span>
        </div>

        {/* 3 Benchmark Cards */}
        {BENCHMARKS.map((b, idx) => {
          const itemSpring = spring({
            frame: frame - idx * 12,
            fps,
            config: { damping: 14, stiffness: 120 },
          });

          return (
            <div
              key={idx}
              style={{
                background: "rgba(10, 16, 30, 0.85)",
                border: "1.5px solid rgba(0, 240, 255, 0.35)",
                borderRadius: 20,
                padding: "26px 36px",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                boxShadow: "0 15px 40px rgba(0,0,0,0.7), inset 0 0 20px rgba(0, 240, 255, 0.08)",
                transform: `scale(${Math.max(0, itemSpring)}) translateY(${(1 - Math.max(0, itemSpring)) * 40}px)`,
                backdropFilter: "blur(12px)",
              }}
            >
              <div>
                <div
                  style={{
                    fontSize: 24,
                    fontWeight: 800,
                    color: "#FFFFFF",
                    fontFamily: THEME.fontFamily,
                  }}
                >
                  {b.label}
                </div>
                <div
                  style={{
                    fontSize: 16,
                    color: "#94A3B8",
                    fontFamily: THEME.monoFamily,
                    marginTop: 6,
                  }}
                >
                  Legacy: <span style={{ color: "#EF4444", textDecoration: "line-through" }}>{b.legacy}</span>
                  {"  "}→{"  "}
                  Fastuse: <span style={{ color: "#00F0FF", fontWeight: 700 }}>{b.fast}</span>
                </div>
              </div>

              {/* Speedup Pill */}
              <div
                style={{
                  background: "linear-gradient(135deg, #00F0FF 0%, #3B82F6 100%)",
                  color: "#030712",
                  padding: "12px 24px",
                  borderRadius: 12,
                  fontFamily: THEME.monoFamily,
                  fontSize: 18,
                  fontWeight: 900,
                  boxShadow: "0 0 25px rgba(0, 240, 255, 0.6)",
                  letterSpacing: "0.05em",
                }}
              >
                {b.speedup}
              </div>
            </div>
          );
        })}
      </div>

      {/* Bottom Kinetic Typography */}
      <div style={{ zIndex: 20, textAlign: "center", marginBottom: 120 }}>
        <KineticWords
          words={[
            { text: "ENTER" },
            { text: "FASTUSE." },
            { text: "PURE" },
            { text: "RUST", color: "#00F0FF", isAccent: true },
            { text: "ENGINE." },
          ]}
          fontSize={64}
          stagger={3}
        />
        <div
          style={{
            marginTop: 16,
            fontSize: 22,
            fontFamily: THEME.monoFamily,
            color: "rgba(255, 255, 255, 0.8)",
            letterSpacing: "0.08em",
          }}
        >
          Zero latency · Native OS APIs · 485x Faster
        </div>
      </div>
    </div>
  );
};
