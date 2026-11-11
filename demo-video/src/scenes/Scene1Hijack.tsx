import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { THEME } from "../theme";

export const Scene1Hijack: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Violent cursor jump simulation
  const cursorX = interpolate(
    frame,
    [0, 60, 120, 180, 240, 300, 360, 420],
    [900, 2400, 800, 3100, 1400, 2700, 950, 2000],
    { extrapolateRight: "clamp" }
  );
  const cursorY = interpolate(
    frame,
    [0, 60, 120, 180, 240, 300, 360, 420],
    [700, 1400, 1800, 600, 1500, 850, 1600, 1100],
    { extrapolateRight: "clamp" }
  );

  // Red glitch flash
  const glitch = Math.sin(frame * 0.4) > 0.6;
  const timer = Math.min(42.8, (frame / fps) * 5.3).toFixed(1);

  const titleSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 120 },
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
      }}
    >
      {/* Background Dark Grid with Red Glitch Pulse */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `linear-gradient(to right, rgba(239, 68, 68, ${glitch ? 0.08 : 0.03}) 2px, transparent 2px), linear-gradient(to bottom, rgba(239, 68, 68, ${glitch ? 0.08 : 0.03}) 2px, transparent 2px)`,
          backgroundSize: "120px 120px",
        }}
      />

      {/* Warning Banner */}
      <div
        style={{
          position: "absolute",
          top: 100,
          display: "flex",
          alignItems: "center",
          gap: 20,
          background: "rgba(239, 68, 68, 0.12)",
          border: "2px solid rgba(239, 68, 68, 0.4)",
          padding: "16px 48px",
          borderRadius: 999,
          transform: `scale(${titleSpring})`,
          opacity: titleSpring,
        }}
      >
        <div
          style={{
            width: 18,
            height: 18,
            borderRadius: "50%",
            background: THEME.red,
            boxShadow: `0 0 16px ${THEME.red}`,
          }}
        />
        <span
          style={{
            fontFamily: THEME.monoFamily,
            fontSize: 32,
            fontWeight: 800,
            color: THEME.red,
            letterSpacing: 2,
            textTransform: "uppercase",
          }}
        >
          CRITICAL FLAW: LEGACY COMPUTER USE
        </span>
      </div>

      {/* Main Kinetic Typography */}
      <div
        style={{
          textAlign: "center",
          display: "flex",
          flexDirection: "column",
          gap: 28,
          zIndex: 10,
        }}
      >
        <h1
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 140,
            fontWeight: 900,
            color: THEME.text,
            lineHeight: 1.1,
            margin: 0,
            letterSpacing: -2,
            textShadow: "0 10px 40px rgba(0,0,0,0.8)",
          }}
        >
          YOUR MOUSE IS <span style={{ color: THEME.red }}>HIJACKED.</span>
        </h1>
        <h2
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 72,
            fontWeight: 700,
            color: THEME.textMuted,
            margin: 0,
          }}
        >
          Screen frozen. Focus locked. Your workflow stops.
        </h2>
      </div>

      {/* Latency Counter Overlay Card */}
      <div
        style={{
          marginTop: 80,
          background: "rgba(14, 19, 31, 0.8)",
          border: "2px solid rgba(239, 68, 68, 0.3)",
          borderRadius: 24,
          padding: "36px 72px",
          display: "flex",
          alignItems: "center",
          gap: 40,
          boxShadow: "0 20px 60px rgba(0,0,0,0.6)",
        }}
      >
        <span
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 36,
            fontWeight: 700,
            color: THEME.textMuted,
          }}
        >
          Time Burned Per Action:
        </span>
        <span
          style={{
            fontFamily: THEME.monoFamily,
            fontSize: 64,
            fontWeight: 900,
            color: THEME.red,
            textShadow: "0 0 24px rgba(239, 68, 68, 0.5)",
          }}
        >
          {timer}s
        </span>
        <span
          style={{
            fontFamily: THEME.monoFamily,
            fontSize: 28,
            color: THEME.red,
            background: "rgba(239, 68, 68, 0.15)",
            padding: "8px 20px",
            borderRadius: 8,
          }}
        >
          TIMEOUT WARNING
        </span>
      </div>

      {/* Erratic OS White Mouse Pointer (Simulating Hijack) */}
      <div
        style={{
          position: "absolute",
          left: cursorX,
          top: cursorY,
          pointerEvents: "none",
          filter: "drop-shadow(0 8px 16px rgba(0,0,0,0.8))",
          transform: "scale(2.2)",
        }}
      >
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
          <path
            d="M5 3L19 12L12 14L9 21L5 3Z"
            fill="#FFFFFF"
            stroke="#000000"
            strokeWidth="1.5"
          />
        </svg>
        <div
          style={{
            position: "absolute",
            left: 28,
            top: 28,
            background: THEME.red,
            color: "#FFF",
            fontFamily: THEME.monoFamily,
            fontSize: 12,
            fontWeight: 800,
            padding: "4px 8px",
            borderRadius: 4,
            whiteSpace: "nowrap",
          }}
        >
          AI CONTROLLING OS POINTER
        </div>
      </div>
    </div>
  );
};
