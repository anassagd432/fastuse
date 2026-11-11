import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";
import { THEME } from "../theme";

export const Scene6Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const heroSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 100 },
  });

  const buttonSpring = spring({
    frame: frame - 20,
    fps,
    config: { damping: 12, stiffness: 120 },
  });

  const starPulse = Math.sin(frame * 0.1) * 0.05 + 1.0;

  return (
    <div
      style={{
        width: 3840,
        height: 2160,
        background: THEME.bg,
        position: "relative",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 50,
      }}
    >
      {/* Background Pulsing Cyan Glow */}
      <div
        style={{
          position: "absolute",
          width: 2600,
          height: 2600,
          borderRadius: "50%",
          background: "radial-gradient(circle, rgba(47, 107, 255, 0.22) 0%, transparent 70%)",
          filter: "blur(90px)",
          pointerEvents: "none",
        }}
      />

      {/* Main Kinetic Headline */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 20,
          transform: `scale(${heroSpring})`,
          opacity: heroSpring,
        }}
      >
        <h2
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 72,
            fontWeight: 800,
            color: THEME.textMuted,
            margin: 0,
            letterSpacing: 2,
            textTransform: "uppercase",
          }}
        >
          STOP WAITING 40 SECONDS.
        </h2>
        <h1
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 160,
            fontWeight: 900,
            color: THEME.text,
            letterSpacing: -3,
            margin: 0,
            textShadow: `0 0 60px ${THEME.blueGlow}`,
          }}
        >
          TAKE BACK YOUR <span style={{ color: THEME.cyan }}>CURSOR.</span>
        </h1>
      </div>

      {/* Hero Repo Lockup Card */}
      <div
        style={{
          background: "rgba(14, 19, 31, 0.95)",
          border: `2px solid ${THEME.blue}`,
          borderRadius: 36,
          padding: "50px 100px",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 36,
          boxShadow: `0 30px 100px rgba(0, 0, 0, 0.9), 0 0 60px ${THEME.blueGlow}`,
          transform: `scale(${buttonSpring * starPulse})`,
          opacity: buttonSpring,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
          <span style={{ fontSize: 56 }}>⚡</span>
          <span
            style={{
              fontFamily: THEME.fontFamily,
              fontSize: 64,
              fontWeight: 900,
              color: THEME.text,
            }}
          >
            fastuse
          </span>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 24,
              color: THEME.cyan,
              border: `1.5px solid ${THEME.cyan}`,
              padding: "6px 18px",
              borderRadius: 999,
              textTransform: "uppercase",
            }}
          >
            Open Source · MIT
          </span>
        </div>

        <span
          style={{
            fontFamily: THEME.monoFamily,
            fontSize: 40,
            fontWeight: 700,
            color: THEME.blueLight,
            letterSpacing: 1,
          }}
        >
          github.com/anassagd432/fastuse
        </span>

        {/* GitHub Star CTA Button */}
        <div
          style={{
            background: `linear-gradient(135deg, ${THEME.blue}, #1D4ED8)`,
            borderRadius: 20,
            padding: "24px 64px",
            display: "flex",
            alignItems: "center",
            gap: 20,
            boxShadow: `0 12px 40px ${THEME.blueGlow}`,
          }}
        >
          <span style={{ fontSize: 36 }}>⭐</span>
          <span
            style={{
              fontFamily: THEME.fontFamily,
              fontSize: 36,
              fontWeight: 800,
              color: "#FFFFFF",
              letterSpacing: 0.5,
            }}
          >
            Star on GitHub
          </span>
        </div>
      </div>

      {/* Platform Support Badges */}
      <div
        style={{
          display: "flex",
          gap: 28,
          alignItems: "center",
          marginTop: 20,
        }}
      >
        <span style={{ fontFamily: THEME.monoFamily, fontSize: 24, color: THEME.textMuted }}>
          Supported Platforms:
        </span>
        {["Windows 11", "macOS Sonoma/Sequoia", "Brave", "Chrome", "Edge", "Google Antigravity"].map((p) => (
          <div
            key={p}
            style={{
              background: "rgba(255, 255, 255, 0.06)",
              border: "1px solid rgba(255, 255, 255, 0.12)",
              borderRadius: 12,
              padding: "10px 22px",
              fontFamily: THEME.fontFamily,
              fontSize: 22,
              fontWeight: 700,
              color: THEME.text,
            }}
          >
            {p}
          </div>
        ))}
      </div>
    </div>
  );
};
