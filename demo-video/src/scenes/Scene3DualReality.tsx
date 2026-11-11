import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { AgentCursor } from "../components/AgentCursor";
import { GlowFrame } from "../components/GlowFrame";
import { THEME } from "../theme";

export const Scene3DualReality: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const sceneProgress = spring({
    frame,
    fps,
    config: { damping: 16, stiffness: 100 },
  });

  // Typing animation for user's VS Code window
  const codeLines = [
    "// You keep typing uninterrupted...",
    "export async function processTask() {",
    "  const stream = await engine.connect();",
    "  return stream.execute({ parallel: true });",
    "}",
  ];
  const typedCount = Math.floor(interpolate(frame, [0, 400], [0, 140], { extrapolateRight: "clamp" }));
  const fullText = codeLines.join("\n");
  const visibleText = fullText.slice(0, typedCount);

  // Agent cursor trajectory on the right background window
  const agentX = interpolate(frame, [0, 120, 240, 360], [2200, 2600, 3050, 3050], { extrapolateRight: "clamp" });
  const agentY = interpolate(frame, [0, 120, 240, 360], [1300, 1100, 1420, 1420], { extrapolateRight: "clamp" });

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
      {/* Screen Edge Aura */}
      <GlowFrame bannerText="This computer is being controlled by fastuse" />

      {/* Header Banner */}
      <div
        style={{
          position: "absolute",
          top: 150,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 16,
          zIndex: 10,
        }}
      >
        <span
          style={{
            fontFamily: THEME.monoFamily,
            fontSize: 28,
            fontWeight: 800,
            color: THEME.cyan,
            letterSpacing: 2,
            textTransform: "uppercase",
            background: "rgba(47, 107, 255, 0.15)",
            padding: "8px 24px",
            borderRadius: 999,
            border: "1px solid rgba(47, 107, 255, 0.4)",
          }}
        >
          DUAL CONCURRENCY WITHOUT COLLISION
        </span>
        <h2
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 84,
            fontWeight: 900,
            color: THEME.text,
            margin: 0,
            letterSpacing: -1,
          }}
        >
          YOUR MOUSE STAYS <span style={{ color: THEME.cyan }}>100% FREE.</span>
        </h2>
      </div>

      {/* Split Screen Cards Container */}
      <div
        style={{
          display: "flex",
          gap: 60,
          width: 3300,
          height: 1400,
          marginTop: 220,
          transform: `scale(${sceneProgress})`,
          opacity: sceneProgress,
        }}
      >
        {/* Left Window: User Coding (VS Code) */}
        <div
          style={{
            flex: 1,
            background: "rgba(15, 23, 42, 0.95)",
            border: "2px solid rgba(255, 255, 255, 0.12)",
            borderRadius: 28,
            boxShadow: "0 24px 60px rgba(0, 0, 0, 0.7)",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
            position: "relative",
          }}
        >
          {/* Window Header */}
          <div
            style={{
              height: 70,
              background: "#0F172A",
              borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
              display: "flex",
              alignItems: "center",
              padding: "0 28px",
              justifyContent: "space-between",
            }}
          >
            <div style={{ display: "flex", gap: 12 }}>
              <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#EF4444" }} />
              <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#F59E0B" }} />
              <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#10B981" }} />
            </div>
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 24, fontWeight: 700, color: THEME.textMuted }}>
              VS Code — main.ts (User Active)
            </span>
            <span style={{ fontFamily: THEME.monoFamily, fontSize: 20, color: THEME.emerald }}>
              ● USER TYPING
            </span>
          </div>

          {/* Code Body */}
          <div style={{ padding: "48px 60px", flex: 1 }}>
            <pre
              style={{
                fontFamily: THEME.monoFamily,
                fontSize: 34,
                lineHeight: 1.6,
                color: THEME.text,
                margin: 0,
                whiteSpace: "pre-wrap",
              }}
            >
              {visibleText}
              <span style={{ borderLeft: `3px solid ${THEME.cyan}`, marginLeft: 4 }} />
            </pre>
          </div>

          {/* User's White Mouse Resting in Window */}
          <div
            style={{
              position: "absolute",
              bottom: 120,
              right: 140,
              transform: "scale(1.8)",
              filter: "drop-shadow(0 6px 12px rgba(0,0,0,0.5))",
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <path d="M5 3L19 12L12 14L9 21L5 3Z" fill="#FFFFFF" stroke="#000000" strokeWidth="1.5" />
            </svg>
            <span
              style={{
                position: "absolute",
                left: 28,
                top: 24,
                fontFamily: THEME.monoFamily,
                fontSize: 14,
                color: "#CBD5E1",
                background: "rgba(0,0,0,0.6)",
                padding: "2px 8px",
                borderRadius: 4,
                whiteSpace: "nowrap",
              }}
            >
              Your Real Cursor
            </span>
          </div>
        </div>

        {/* Right Window: Background Operation (Notepad / Settings) */}
        <div
          style={{
            flex: 1,
            background: "rgba(14, 19, 31, 0.95)",
            border: `2px solid ${THEME.blue}`,
            borderRadius: 28,
            boxShadow: `0 24px 60px rgba(0, 0, 0, 0.7), 0 0 40px ${THEME.blueGlow}`,
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
            position: "relative",
          }}
        >
          {/* Window Header */}
          <div
            style={{
              height: 70,
              background: "#1E293B",
              borderBottom: "1px solid rgba(47, 107, 255, 0.3)",
              display: "flex",
              alignItems: "center",
              padding: "0 28px",
              justifyContent: "space-between",
            }}
          >
            <div style={{ display: "flex", gap: 12 }}>
              <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#475569" }} />
              <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#475569" }} />
              <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#475569" }} />
            </div>
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 24, fontWeight: 700, color: THEME.cyan }}>
              Background App (fastuse Win32/AX)
            </span>
            <span style={{ fontFamily: THEME.monoFamily, fontSize: 20, color: THEME.cyan }}>
              ● AGENT ACTING
            </span>
          </div>

          {/* Background Window Content */}
          <div style={{ padding: "48px 60px", display: "flex", flexDirection: "column", gap: 40, flex: 1 }}>
            <div
              style={{
                fontFamily: THEME.fontFamily,
                fontSize: 32,
                fontWeight: 600,
                color: THEME.textMuted,
              }}
            >
              Deployment Configuration
            </div>

            <div
              style={{
                background: "rgba(255, 255, 255, 0.05)",
                border: "1px solid rgba(255, 255, 255, 0.1)",
                borderRadius: 16,
                padding: "24px 32px",
                fontFamily: THEME.monoFamily,
                fontSize: 28,
                color: THEME.text,
              }}
            >
              Environment: production-cluster-east
            </div>

            <div style={{ display: "flex", gap: 24, marginTop: "auto" }}>
              <div
                style={{
                  background: "rgba(255, 255, 255, 0.08)",
                  borderRadius: 16,
                  padding: "24px 48px",
                  fontFamily: THEME.fontFamily,
                  fontSize: 28,
                  fontWeight: 700,
                  color: THEME.textMuted,
                }}
              >
                Cancel
              </div>
              <div
                style={{
                  background: THEME.blue,
                  borderRadius: 16,
                  padding: "24px 56px",
                  fontFamily: THEME.fontFamily,
                  fontSize: 28,
                  fontWeight: 700,
                  color: "#FFFFFF",
                  boxShadow: `0 0 30px ${THEME.blueGlow}`,
                }}
              >
                Confirm & Deploy
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Agent Cursor with Click Ripple */}
      <AgentCursor x={agentX} y={agentY} clickFrame={240} scale={2.4} />
    </div>
  );
};
