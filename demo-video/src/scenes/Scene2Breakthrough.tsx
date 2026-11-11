import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";
import { BenchmarkCard } from "../components/BenchmarkCard";
import { THEME } from "../theme";

export const Scene2Breakthrough: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoSpring = spring({
    frame,
    fps,
    config: { damping: 12, stiffness: 100 },
  });

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
        gap: 60,
      }}
    >
      {/* Background Radial Blue Light */}
      <div
        style={{
          position: "absolute",
          width: 2400,
          height: 2400,
          borderRadius: "50%",
          background: "radial-gradient(circle, rgba(47, 107, 255, 0.16) 0%, transparent 70%)",
          filter: "blur(80px)",
          pointerEvents: "none",
        }}
      />

      {/* Hero Header Lockup */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 20,
          transform: `scale(${logoSpring})`,
          opacity: logoSpring,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 24,
            background: "rgba(47, 107, 255, 0.15)",
            border: `2px solid ${THEME.blue}`,
            padding: "14px 44px",
            borderRadius: 999,
            boxShadow: `0 0 30px ${THEME.blueGlow}`,
          }}
        >
          <span style={{ fontSize: 32 }}>⚡</span>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 32,
              fontWeight: 800,
              color: THEME.cyan,
              letterSpacing: 2,
              textTransform: "uppercase",
            }}
          >
            NOT ANYMORE · BUILT IN NATIVE RUST
          </span>
        </div>

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
          MEET <span style={{ color: THEME.cyan }}>fastuse</span>
        </h1>
      </div>

      {/* Benchmark Telemetry Card */}
      <BenchmarkCard />
    </div>
  );
};
