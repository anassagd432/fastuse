import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { TunnelWarp } from "../components/TunnelWarp";
import { StarburstBadge } from "../components/StarburstBadge";
import { CinematicHud } from "../components/CinematicHud";
import { KineticWords } from "../components/KineticWords";
import { THEME } from "../theme";

const INTEGRATION_CHECKS = [
  { name: "Antigravity MCP Integration", status: "NATIVE & AUTOMATIC", latency: "0.2ms" },
  { name: "Claude Code CLI MCP Bridge", status: "ONE-CLICK READY", latency: "0.4ms" },
  { name: "Background Desktop Controller", status: "UNBLOCKED RUNTIME", latency: "144 FPS" },
  { name: "Brave & Chrome Live Tab Bridge", status: "COOKIE INHERITED", latency: "0.9ms" },
];

export const VerticalScene5Integration: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const cardSpring = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 120 },
  });

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
      {/* 3D Tunnel Warp Background in Electric Purple & Blue */}
      <TunnelWarp speed={1.5} glowColor="#8A2BE2" />

      {/* Cinematic HUD */}
      <CinematicHud
        sceneIndex="05 / 06"
        sceneTitle="ANTIGRAVITY & CLAUDE CODE NATIVE BRIDGE"
        fpsText="144.0 FPS"
        statusText="ZERO CONFIG MCP PIPELINE"
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
          text="ONE PROMPT"
          subtext="AUTO INSTALL"
          color="#8A2BE2"
          textColor="#FFFFFF"
          size={195}
          rotationOffset={10}
        />
      </div>

      {/* Center 3D Integration Terminal Cockpit */}
      <div
        style={{
          zIndex: 20,
          width: "100%",
          maxWidth: 960,
          marginTop: 220,
          display: "flex",
          flexDirection: "column",
          gap: 22,
          transform: `scale(${cardSpring})`,
        }}
      >
        {/* Logos Header Pill */}
        <div
          style={{
            alignSelf: "center",
            display: "flex",
            alignItems: "center",
            gap: 16,
            background: "rgba(138, 43, 226, 0.2)",
            border: "2px solid #8A2BE2",
            boxShadow: "0 0 35px rgba(138, 43, 226, 0.45)",
            padding: "12px 36px",
            borderRadius: 999,
          }}
        >
          <span style={{ fontSize: 24 }}>🚀</span>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 18,
              fontWeight: 800,
              color: "#C084FC",
              letterSpacing: "0.12em",
            }}
          >
            ANTIGRAVITY + CLAUDE CODE READY
          </span>
        </div>

        {/* Holographic Prompt Paste Terminal */}
        <div
          style={{
            background: "#080C18",
            border: "2px solid rgba(138, 43, 226, 0.5)",
            borderRadius: 20,
            padding: "24px 28px",
            boxShadow: "0 20px 50px rgba(0,0,0,0.8), inset 0 0 20px rgba(138, 43, 226, 0.15)",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              borderBottom: "1px solid rgba(255,255,255,0.1)",
              paddingBottom: 10,
              marginBottom: 14,
            }}
          >
            <div style={{ display: "flex", gap: 6 }}>
              <div style={{ width: 10, height: 10, borderRadius: "50%", backgroundColor: "#EF4444" }} />
              <div style={{ width: 10, height: 10, borderRadius: "50%", backgroundColor: "#F59E0B" }} />
              <div style={{ width: 10, height: 10, borderRadius: "50%", backgroundColor: "#10B981" }} />
            </div>
            <span style={{ fontSize: 12, fontFamily: THEME.monoFamily, color: "#C084FC" }}>
              PASTE INTO YOUR AI ASSISTANT:
            </span>
          </div>

          <div
            style={{
              background: "#030611",
              border: "1px dashed rgba(192, 132, 252, 0.4)",
              borderRadius: 12,
              padding: "14px 18px",
              fontFamily: THEME.monoFamily,
              fontSize: 15,
              color: "#E2E8F0",
              lineHeight: 1.5,
            }}
          >
            <span style={{ color: "#F472B6" }}>Install</span> fastuse MCP engine and configure it as my{" "}
            <span style={{ color: "#38BDF8", fontWeight: 700 }}>default background computer use tool</span>.
          </div>
        </div>

        {/* 4 Integration Verification Cards */}
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          {INTEGRATION_CHECKS.map((item, idx) => {
            const checkSpring = spring({
              frame: frame - idx * 8,
              fps,
              config: { damping: 14, stiffness: 120 },
            });

            return (
              <div
                key={idx}
                style={{
                  background: "rgba(10, 16, 32, 0.85)",
                  border: "1px solid rgba(16, 185, 129, 0.35)",
                  borderRadius: 14,
                  padding: "16px 24px",
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  transform: `scale(${Math.max(0, checkSpring)})`,
                  backdropFilter: "blur(12px)",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                  <div
                    style={{
                      width: 28,
                      height: 28,
                      borderRadius: "50%",
                      backgroundColor: "rgba(16, 185, 129, 0.2)",
                      border: "1.5px solid #10B981",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      color: "#10B981",
                      fontWeight: 900,
                      fontSize: 14,
                    }}
                  >
                    ✓
                  </div>
                  <span style={{ fontSize: 17, fontWeight: 700, color: "#FFFFFF", fontFamily: THEME.fontFamily }}>
                    {item.name}
                  </span>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <span
                    style={{
                      fontFamily: THEME.monoFamily,
                      fontSize: 13,
                      fontWeight: 800,
                      color: "#10B981",
                    }}
                  >
                    {item.status}
                  </span>
                  <span
                    style={{
                      fontFamily: THEME.monoFamily,
                      fontSize: 12,
                      color: "#94A3B8",
                      background: "#1E293B",
                      padding: "2px 8px",
                      borderRadius: 4,
                    }}
                  >
                    {item.latency}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Bottom Kinetic Typography */}
      <div style={{ zIndex: 20, textAlign: "center", marginBottom: 120 }}>
        <KineticWords
          words={[
            { text: "ONE" },
            { text: "PROMPT." },
            { text: "ZERO" },
            { text: "EFFORT.", color: "#8A2BE2", isAccent: true },
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
          Just copy-paste into Antigravity or Claude Code
        </div>
      </div>
    </div>
  );
};
