import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { HoloLaptop } from "../components/HoloLaptop";
import { StarburstBadge } from "../components/StarburstBadge";
import { CinematicHud } from "../components/CinematicHud";
import { KineticWords } from "../components/KineticWords";
import { THEME } from "../theme";

export const VerticalScene1Hijack: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Violent cursor jumps across the screen
  const cursorX = interpolate(
    frame,
    [0, 40, 90, 140, 200, 260, 320],
    [120, 520, 210, 600, 340, 580, 280],
    { extrapolateRight: "clamp" }
  );
  const cursorY = interpolate(
    frame,
    [0, 40, 90, 140, 200, 260, 320],
    [80, 320, 150, 380, 220, 110, 290],
    { extrapolateRight: "clamp" }
  );

  const glitchActive = Math.sin(frame * 0.45) > 0.6;
  const shake = glitchActive ? (Math.sin(frame * 1.5) * 4) : 0;

  const timer = Math.min(42.8, (frame / fps) * 7.5).toFixed(1);

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
        transform: `translate(${shake}px, ${shake * 0.5}px)`,
      }}
    >
      {/* Background Red Ambient Glow */}
      <div
        style={{
          position: "absolute",
          top: "35%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          width: 600,
          height: 600,
          borderRadius: "50%",
          background: "radial-gradient(circle, rgba(239, 68, 68, 0.22) 0%, transparent 70%)",
          filter: "blur(70px)",
          pointerEvents: "none",
        }}
      />

      {/* Cinematic HUD Overlay */}
      <CinematicHud
        sceneIndex="01 / 06"
        sceneTitle="CRITICAL FRICTION: THE CURSOR HIJACK"
        fpsText="12.0 FPS (STALLED)"
        statusText="SYSTEM BLOCKED BY LEGACY TOOL"
      />

      {/* Floating Starburst Warning Badge */}
      <div
        style={{
          position: "absolute",
          top: 140,
          right: 70,
          zIndex: 30,
        }}
      >
        <StarburstBadge
          text="CURSOR HIJACKED"
          subtext="SCREEN FROZEN"
          color="#EF4444"
          size={190}
          rotationOffset={12}
        />
      </div>

      {/* Center 3D Laptop Showcase with Reflection */}
      <div style={{ marginTop: 200, zIndex: 15 }}>
        <HoloLaptop>
          {/* Inner Screen Display (Operating system chaos) */}
          <div
            style={{
              width: "100%",
              height: "100%",
              backgroundColor: "#070B14",
              position: "relative",
              overflow: "hidden",
              padding: 24,
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
            }}
          >
            {/* Top window bar */}
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                borderBottom: "1px solid rgba(239,68,68,0.3)",
                paddingBottom: 8,
              }}
            >
              <div style={{ display: "flex", gap: 6 }}>
                <div style={{ width: 10, height: 10, borderRadius: "50%", backgroundColor: "#EF4444" }} />
                <div style={{ width: 10, height: 10, borderRadius: "50%", backgroundColor: "#F59E0B" }} />
                <div style={{ width: 10, height: 10, borderRadius: "50%", backgroundColor: "#10B981" }} />
              </div>
              <span style={{ fontSize: 11, fontFamily: THEME.monoFamily, color: "#EF4444", fontWeight: 700 }}>
                ⚠ SYSTEM STEAL IN PROGRESS (SCREEN LOCKED)
              </span>
            </div>

            {/* Red Alert Card in Center */}
            <div
              style={{
                background: "rgba(239, 68, 68, 0.12)",
                border: "2px solid #EF4444",
                borderRadius: 12,
                padding: "20px 24px",
                boxShadow: "0 0 30px rgba(239, 68, 68, 0.4)",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 8 }}>
                <span style={{ fontSize: 24 }}>🚫</span>
                <span style={{ fontSize: 18, fontWeight: 900, color: "#EF4444", fontFamily: THEME.fontFamily }}>
                  DO NOT TOUCH YOUR MOUSE
                </span>
              </div>
              <p style={{ fontSize: 13, color: "#FCA5A5", margin: 0, fontFamily: THEME.monoFamily }}>
                Legacy agent is moving your physical cursor. Your workstation is completely frozen.
              </p>
              {/* Progress bar */}
              <div
                style={{
                  marginTop: 14,
                  height: 8,
                  backgroundColor: "rgba(0,0,0,0.6)",
                  borderRadius: 4,
                  overflow: "hidden",
                }}
              >
                <div
                  style={{
                    height: "100%",
                    width: `${Math.min(95, frame * 0.25)}%`,
                    backgroundColor: "#EF4444",
                    boxShadow: "0 0 10px #EF4444",
                  }}
                />
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", marginTop: 6, fontSize: 11, color: "#94A3B8" }}>
                <span>Action: slow screenshot poll</span>
                <span style={{ color: "#EF4444", fontWeight: 700 }}>Wait time: {timer}s</span>
              </div>
            </div>

            {/* Wildly jumping cursor inside screen */}
            <div
              style={{
                position: "absolute",
                top: cursorY,
                left: cursorX,
                pointerEvents: "none",
                transform: "translate(-50%, -50%)",
                filter: "drop-shadow(0 0 10px #EF4444)",
              }}
            >
              <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
                <path d="M3 3L10 21L13 14L20 11L3 3Z" fill="#EF4444" stroke="#FFFFFF" strokeWidth="1.5" />
              </svg>
            </div>
          </div>
        </HoloLaptop>
      </div>

      {/* Bottom Kinetic Typography */}
      <div style={{ zIndex: 20, textAlign: "center", marginBottom: 120 }}>
        <KineticWords
          words={[
            { text: "EVERY" },
            { text: "AI" },
            { text: "AGENT" },
            { text: "HITS" },
            { text: "A" },
            { text: "WALL.", color: "#EF4444", isAccent: true },
          ]}
          fontSize={68}
          stagger={3}
        />
        <div
          style={{
            marginTop: 16,
            fontSize: 22,
            fontFamily: THEME.monoFamily,
            color: "rgba(255, 255, 255, 0.7)",
            letterSpacing: "0.08em",
          }}
        >
          Takes your mouse · Steals your focus · Freezes your screen
        </div>
      </div>
    </div>
  );
};
