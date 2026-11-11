import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { MeshWave3D } from "../components/MeshWave3D";
import { StarburstBadge } from "../components/StarburstBadge";
import { CinematicHud } from "../components/CinematicHud";
import { KineticWords } from "../components/KineticWords";
import { THEME } from "../theme";

export const VerticalScene6Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Entrance spring
  const outroSpring = spring({
    frame,
    fps,
    config: { damping: 12, stiffness: 130 },
  });

  // Pulsing star rotation and scale
  const starScale = 1.0 + Math.sin(frame * 0.1) * 0.08;

  // GitHub star count ticker
  const starCount = Math.floor(interpolate(frame, [0, 180], [120, 10842], { extrapolateRight: "clamp" }));

  return (
    <div
      style={{
        width: 1080,
        height: 1920,
        backgroundColor: "#030611",
        position: "relative",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "80px 40px",
      }}
    >
      {/* 3D Undulating Mesh Wave Background */}
      <MeshWave3D cols={24} rows={24} waveSpeed={1.4} colorScheme="neon" />

      {/* Cinematic HUD */}
      <CinematicHud
        sceneIndex="06 / 06"
        sceneTitle="OPEN SOURCE REVOLUTION // STAR THE REPO"
        fpsText="144.0 FPS"
        statusText="FASTUSE // 100% OPEN SOURCE"
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
          text="STAR NOW"
          subtext="GITHUB"
          color="#FFB800"
          textColor="#030712"
          size={200}
          rotationOffset={-8}
        />
      </div>

      {/* Center Hero Outro Card */}
      <div
        style={{
          zIndex: 20,
          width: "100%",
          maxWidth: 960,
          marginTop: 220,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 28,
          transform: `scale(${outroSpring})`,
        }}
      >
        {/* Giant Logo Badge */}
        <div
          style={{
            width: 110,
            height: 110,
            borderRadius: 30,
            background: "linear-gradient(135deg, #00F0FF 0%, #3B82F6 50%, #8A2BE2 100%)",
            boxShadow: "0 0 60px rgba(0, 240, 255, 0.65), 0 20px 40px rgba(0,0,0,0.8)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 54,
            transform: `scale(${starScale})`,
          }}
        >
          ⚡
        </div>

        {/* Title */}
        <div
          style={{
            fontSize: 88,
            fontWeight: 900,
            color: "#FFFFFF",
            fontFamily: THEME.fontFamily,
            letterSpacing: "-0.04em",
            textAlign: "center",
            textShadow: "0 0 40px rgba(0, 240, 255, 0.6), 0 10px 30px rgba(0,0,0,0.9)",
            lineHeight: 1.0,
          }}
        >
          FASTUSE
        </div>

        <div
          style={{
            fontSize: 24,
            fontWeight: 700,
            color: "#94A3B8",
            fontFamily: THEME.monoFamily,
            letterSpacing: "0.15em",
            textTransform: "uppercase",
            textAlign: "center",
          }}
        >
          The Speed Your Agents Actually Deserve
        </div>

        {/* GITHUB STAR BUTTON (MASSIVE CALL TO ACTION) */}
        <div
          style={{
            marginTop: 10,
            background: "linear-gradient(135deg, #FFB800 0%, #F59E0B 100%)",
            borderRadius: 24,
            padding: "24px 50px",
            display: "flex",
            alignItems: "center",
            gap: 20,
            boxShadow:
              "0 0 50px rgba(255, 184, 0, 0.6), 0 20px 40px rgba(0,0,0,0.8), inset 0 2px 4px rgba(255,255,255,0.4)",
            cursor: "pointer",
            transform: `scale(${starScale})`,
          }}
        >
          <span style={{ fontSize: 36, color: "#030712" }}>★</span>
          <div style={{ display: "flex", flexDirection: "column" }}>
            <span
              style={{
                fontSize: 28,
                fontWeight: 900,
                color: "#030712",
                fontFamily: THEME.fontFamily,
                letterSpacing: "-0.02em",
              }}
            >
              STAR ON GITHUB
            </span>
            <span
              style={{
                fontSize: 15,
                fontWeight: 700,
                color: "rgba(3, 7, 18, 0.8)",
                fontFamily: THEME.monoFamily,
              }}
            >
              {starCount.toLocaleString()} DEVELOPERS
            </span>
          </div>
        </div>

        {/* REPO URL CARD */}
        <div
          style={{
            background: "rgba(10, 16, 32, 0.9)",
            border: "1.5px solid rgba(0, 240, 255, 0.4)",
            borderRadius: 16,
            padding: "16px 36px",
            display: "flex",
            alignItems: "center",
            gap: 14,
            boxShadow: "0 10px 30px rgba(0,0,0,0.8)",
          }}
        >
          <span style={{ fontSize: 20, color: "#00F0FF" }}>🔗</span>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 20,
              fontWeight: 800,
              color: "#E2E8F0",
              letterSpacing: "0.05em",
            }}
          >
            github.com/anassagd432/fastuse
          </span>
        </div>

        {/* Feature badges row */}
        <div style={{ display: "flex", gap: 12, marginTop: 4 }}>
          {["PURE RUST", "WINDOWS & MACOS", "ANTIGRAVITY NATIVE", "100% FREE"].map((tag, i) => (
            <span
              key={i}
              style={{
                fontFamily: THEME.monoFamily,
                fontSize: 13,
                fontWeight: 800,
                color: "#94A3B8",
                background: "#080C18",
                border: "1px solid #1E293B",
                padding: "6px 14px",
                borderRadius: 8,
              }}
            >
              {tag}
            </span>
          ))}
        </div>
      </div>

      {/* Bottom Kinetic Typography */}
      <div style={{ zIndex: 20, textAlign: "center", marginBottom: 120 }}>
        <KineticWords
          words={[
            { text: "STAR" },
            { text: "THE" },
            { text: "REPO." },
            { text: "BUILD", color: "#FFB800", isAccent: true },
            { text: "FASTER." },
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
          Available now on GitHub · Open Source under MIT
        </div>
      </div>
    </div>
  );
};
