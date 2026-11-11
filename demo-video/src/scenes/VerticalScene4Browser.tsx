import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { ParticleVortex } from "../components/ParticleVortex";
import { StarburstBadge } from "../components/StarburstBadge";
import { CinematicHud } from "../components/CinematicHud";
import { KineticWords } from "../components/KineticWords";
import { THEME } from "../theme";

const BROWSER_PILLARS = [
  {
    icon: "🍪",
    title: "Native Cookies & Active Sessions",
    desc: "Bypasses all login walls. Reads existing authenticated tabs directly.",
    tag: "INSTANT ACCESS",
    color: "#F59E0B",
  },
  {
    icon: "🔮",
    title: "Shadow DOM Piercing",
    desc: "Traverses through closed shadow roots and dynamic canvas components.",
    tag: "ZERO BLIND SPOTS",
    color: "#A855F7",
  },
  {
    icon: "⚡",
    title: "Single Command Automation",
    desc: "browser_act & browser_state via MCP or direct CLI execution.",
    tag: "0.4ms OVERHEAD",
    color: "#00F0FF",
  },
];

export const VerticalScene4Browser: React.FC = () => {
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
      {/* Generative Swirling Particle Vortex Background */}
      <ParticleVortex particleCount={160} coreColor="#F59E0B" />

      {/* Cinematic HUD */}
      <CinematicHud
        sceneIndex="04 / 06"
        sceneTitle="DIRECT BROWSER ENGINE // BRAVE & CHROME"
        fpsText="144.0 FPS"
        statusText="ZERO HEADLESS INSTANCES REQUIRED"
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
          text="ZERO HEADLESS"
          subtext="REAL BROWSER"
          color="#F59E0B"
          textColor="#030712"
          size={195}
          rotationOffset={-6}
        />
      </div>

      {/* Center 3D Browser Features Cockpit */}
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
        {/* Browser Logos Header */}
        <div
          style={{
            alignSelf: "center",
            display: "flex",
            alignItems: "center",
            gap: 16,
            background: "rgba(245, 158, 11, 0.15)",
            border: "2px solid #F59E0B",
            boxShadow: "0 0 35px rgba(245, 158, 11, 0.35)",
            padding: "12px 36px",
            borderRadius: 999,
          }}
        >
          <span style={{ fontSize: 24 }}>🦁</span>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 18,
              fontWeight: 800,
              color: "#F59E0B",
              letterSpacing: "0.12em",
            }}
          >
            BRAVE & CHROME REAL-ENGINE CONTROL
          </span>
          <span style={{ fontSize: 24 }}>🌐</span>
        </div>

        {/* 3 Pillars Cards */}
        {BROWSER_PILLARS.map((p, idx) => {
          const itemSpring = spring({
            frame: frame - idx * 10,
            fps,
            config: { damping: 14, stiffness: 120 },
          });

          return (
            <div
              key={idx}
              style={{
                background: "rgba(10, 16, 30, 0.88)",
                border: `1.5px solid ${p.color}55`,
                borderRadius: 20,
                padding: "24px 30px",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                boxShadow: `0 15px 40px rgba(0,0,0,0.7), 0 0 25px ${p.color}15`,
                transform: `scale(${Math.max(0, itemSpring)})`,
                backdropFilter: "blur(14px)",
              }}
            >
              <div style={{ display: "flex", gap: 18, alignItems: "center" }}>
                <div
                  style={{
                    width: 56,
                    height: 56,
                    borderRadius: 14,
                    backgroundColor: `${p.color}22`,
                    border: `1.5px solid ${p.color}`,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: 26,
                  }}
                >
                  {p.icon}
                </div>
                <div>
                  <div
                    style={{
                      fontSize: 22,
                      fontWeight: 800,
                      color: "#FFFFFF",
                      fontFamily: THEME.fontFamily,
                    }}
                  >
                    {p.title}
                  </div>
                  <div
                    style={{
                      fontSize: 15,
                      color: "#94A3B8",
                      fontFamily: THEME.monoFamily,
                      marginTop: 4,
                      maxWidth: 540,
                    }}
                  >
                    {p.desc}
                  </div>
                </div>
              </div>

              {/* Tag Pill */}
              <div
                style={{
                  background: `${p.color}22`,
                  border: `1px solid ${p.color}`,
                  color: p.color,
                  padding: "8px 18px",
                  borderRadius: 10,
                  fontFamily: THEME.monoFamily,
                  fontSize: 14,
                  fontWeight: 800,
                  letterSpacing: "0.08em",
                }}
              >
                {p.tag}
              </div>
            </div>
          );
        })}

        {/* Live Command Terminal Card */}
        <div
          style={{
            background: "#060A14",
            border: "1px solid rgba(0, 240, 255, 0.4)",
            borderRadius: 16,
            padding: "16px 24px",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            boxShadow: "0 10px 30px rgba(0,0,0,0.8)",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <span style={{ color: "#00F0FF", fontFamily: THEME.monoFamily, fontWeight: 700 }}>$</span>
            <span style={{ color: "#E2E8F0", fontFamily: THEME.monoFamily, fontSize: 16 }}>
              fastuse call browser_act &apos;&#123;&quot;action&quot;:&quot;click&quot;, &quot;ref&quot;:42&#125;&apos;
            </span>
          </div>
          <span
            style={{
              color: "#10B981",
              fontFamily: THEME.monoFamily,
              fontSize: 13,
              fontWeight: 800,
              background: "rgba(16, 185, 129, 0.15)",
              padding: "4px 12px",
              borderRadius: 6,
            }}
          >
            ✓ 0.4ms
          </span>
        </div>
      </div>

      {/* Bottom Kinetic Typography */}
      <div style={{ zIndex: 20, textAlign: "center", marginBottom: 120 }}>
        <KineticWords
          words={[
            { text: "ZERO" },
            { text: "HEADLESS." },
            { text: "REAL" },
            { text: "BROWSER.", color: "#F59E0B", isAccent: true },
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
          Your cookies · Your logins · Full Shadow DOM support
        </div>
      </div>
    </div>
  );
};
