import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { MeshWave3D } from "../components/MeshWave3D";
import { StarburstBadge } from "../components/StarburstBadge";
import { CinematicHud } from "../components/CinematicHud";
import { KineticWords } from "../components/KineticWords";
import { AgentCursor } from "../components/AgentCursor";
import { THEME } from "../theme";

export const VerticalScene3DualReality: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Entrance spring
  const splitSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 110 },
  });

  // Agent cursor clicking animation on background window
  const agentX = interpolate(
    frame,
    [0, 50, 110, 170, 240, 310, 380, 460],
    [100, 320, 240, 420, 180, 360, 220, 340],
    { extrapolateRight: "clamp" }
  );
  const agentY = interpolate(
    frame,
    [0, 50, 110, 170, 240, 310, 380, 460],
    [80, 140, 220, 180, 260, 120, 200, 160],
    { extrapolateRight: "clamp" }
  );

  const isClicking = (frame % 60) > 48;

  return (
    <div
      style={{
        width: 1080,
        height: 1920,
        backgroundColor: "#03060C",
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
      <MeshWave3D cols={22} rows={22} waveSpeed={1.2} colorScheme="cyber" />

      {/* Cinematic HUD */}
      <CinematicHud
        sceneIndex="03 / 06"
        sceneTitle="DUAL REALITY // PARALLEL PIPELINE"
        fpsText="144.0 FPS"
        statusText="BACKGROUND WORKER: UNBLOCKED"
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
          text="ZERO STEAL"
          subtext="PARALLEL OPS"
          color="#10B981"
          textColor="#030712"
          size={190}
          rotationOffset={8}
        />
      </div>

      {/* DUAL WORKSPACE CARDS */}
      <div
        style={{
          zIndex: 20,
          width: "100%",
          maxWidth: 960,
          marginTop: 220,
          display: "flex",
          flexDirection: "column",
          gap: 30,
          transform: `scale(${splitSpring})`,
        }}
      >
        {/* CARD 1: USER'S DESKTOP (FOCUS UNTOUCHED) */}
        <div
          style={{
            background: "rgba(13, 19, 33, 0.88)",
            border: "2px solid rgba(59, 130, 246, 0.4)",
            borderRadius: 20,
            padding: "24px 30px",
            boxShadow: "0 20px 50px rgba(0,0,0,0.8), inset 0 0 20px rgba(59, 130, 246, 0.1)",
            backdropFilter: "blur(14px)",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: 16,
              borderBottom: "1px solid rgba(255,255,255,0.1)",
              paddingBottom: 10,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <span style={{ fontSize: 20 }}>💻</span>
              <span
                style={{
                  fontFamily: THEME.monoFamily,
                  fontSize: 18,
                  fontWeight: 800,
                  color: "#60A5FA",
                }}
              >
                YOU: ACTIVE WORKSPACE (FULL CONTROL)
              </span>
            </div>
            <span
              style={{
                fontFamily: THEME.monoFamily,
                fontSize: 13,
                color: "#10B981",
                fontWeight: 700,
                background: "rgba(16, 185, 129, 0.15)",
                padding: "4px 12px",
                borderRadius: 6,
              }}
            >
              FOCUS INTACT
            </span>
          </div>

          <div
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 15,
              color: "#E2E8F0",
              lineHeight: 1.6,
              background: "#080C16",
              padding: 16,
              borderRadius: 10,
              border: "1px solid #1E293B",
            }}
          >
            <div style={{ color: "#94A3B8" }}>// You write code smoothly without interference</div>
            <div>
              <span style={{ color: "#F472B6" }}>const</span> engine ={" "}
              <span style={{ color: "#38BDF8" }}>await</span> Fastuse.
              <span style={{ color: "#FBBF24" }}>init</span>();
            </div>
            <div>
              engine.<span style={{ color: "#60A5FA" }}>startBackgroundWorker</span>();
            </div>
          </div>
        </div>

        {/* CARD 2: AGENT'S DESKTOP (BACKGROUND AUTOMATION) */}
        <div
          style={{
            background: "rgba(10, 25, 47, 0.92)",
            border: "2px solid #00F0FF",
            borderRadius: 20,
            padding: "24px 30px",
            boxShadow: "0 25px 60px rgba(0, 240, 255, 0.25), inset 0 0 25px rgba(0, 240, 255, 0.12)",
            backdropFilter: "blur(14px)",
            position: "relative",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: 16,
              borderBottom: "1px solid rgba(0, 240, 255, 0.3)",
              paddingBottom: 10,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <span style={{ fontSize: 20 }}>🤖</span>
              <span
                style={{
                  fontFamily: THEME.monoFamily,
                  fontSize: 18,
                  fontWeight: 800,
                  color: "#00F0FF",
                }}
              >
                AGENT: BRAVE AUTOMATION (BACKGROUND)
              </span>
            </div>
            <span
              style={{
                fontFamily: THEME.monoFamily,
                fontSize: 13,
                color: "#00F0FF",
                fontWeight: 700,
                background: "rgba(0, 240, 255, 0.15)",
                padding: "4px 12px",
                borderRadius: 6,
              }}
            >
              0.9ms LATENCY
            </span>
          </div>

          <div
            style={{
              position: "relative",
              height: 200,
              background: "#050912",
              borderRadius: 10,
              border: "1px solid #1E293B",
              overflow: "hidden",
              padding: 16,
            }}
          >
            {/* Fake browser UI elements */}
            <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
              <div
                style={{
                  background: "#1E293B",
                  padding: "6px 14px",
                  borderRadius: 6,
                  fontSize: 12,
                  fontFamily: THEME.monoFamily,
                  color: "#94A3B8",
                }}
              >
                brave://settings
              </div>
              <div
                style={{
                  background: "rgba(0, 240, 255, 0.2)",
                  border: "1px solid #00F0FF",
                  padding: "6px 14px",
                  borderRadius: 6,
                  fontSize: 12,
                  fontFamily: THEME.monoFamily,
                  color: "#00F0FF",
                  fontWeight: 700,
                }}
              >
                github.com/fastuse [ACTIVE]
              </div>
            </div>

            <div style={{ display: "flex", gap: 12, marginTop: 14 }}>
              <div
                style={{
                  flex: 1,
                  height: 60,
                  backgroundColor: "#0F172A",
                  borderRadius: 6,
                  border: "1px solid #334155",
                  padding: 8,
                }}
              >
                <div style={{ fontSize: 11, color: "#64748B", fontFamily: THEME.monoFamily }}>
                  Shadow DOM Node #48
                </div>
                <div style={{ fontSize: 13, color: "#38BDF8", fontWeight: 700 }}>
                  {"<button id='star'>"}
                </div>
              </div>
              <div
                style={{
                  width: 140,
                  height: 60,
                  backgroundColor: isClicking ? "#10B981" : "#0284C7",
                  borderRadius: 6,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontWeight: 800,
                  fontSize: 13,
                  color: "#FFFFFF",
                  fontFamily: THEME.fontFamily,
                  boxShadow: isClicking ? "0 0 20px #10B981" : "none",
                  transition: "all 0.1s ease",
                }}
              >
                {isClicking ? "CLICKED!" : "AUTO-CLICK"}
              </div>
            </div>

            {/* Simulated Live Agent Cursor */}
            <AgentCursor
              x={agentX}
              y={agentY}
              isClicking={isClicking}
              label="fastuse::click"
            />
          </div>
        </div>
      </div>

      {/* Bottom Kinetic Typography */}
      <div style={{ zIndex: 20, textAlign: "center", marginBottom: 120 }}>
        <KineticWords
          words={[
            { text: "ZERO" },
            { text: "CURSOR" },
            { text: "STEALING.", color: "#10B981", isAccent: true },
            { text: "PARALLEL" },
            { text: "OPS." },
          ]}
          fontSize={62}
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
          You work undisturbed · Agent runs at 144 FPS in background
        </div>
      </div>
    </div>
  );
};
