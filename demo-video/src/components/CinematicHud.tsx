import React from "react";
import { useCurrentFrame } from "remotion";

interface CinematicHudProps {
  sceneTitle: string;
  sceneIndex: string;
  fpsText?: string;
  statusText?: string;
}

export const CinematicHud: React.FC<CinematicHudProps> = ({
  sceneTitle,
  sceneIndex,
  fpsText = "144.0 FPS",
  statusText = "FASTUSE // LIVE BACKGROUND ENGINE",
}) => {
  const frame = useCurrentFrame();

  // Timecode calculation
  const totalSeconds = Math.floor(frame / 60);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  const frames = frame % 60;
  const timecode = `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}:${String(frames).padStart(2, "0")}`;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        zIndex: 50,
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Courier New', monospace",
        color: "rgba(255, 255, 255, 0.75)",
        padding: "48px 40px",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
      }}
    >
      {/* CORNER BRACKETS */}
      {/* Top Left */}
      <div
        style={{
          position: "absolute",
          top: 36,
          left: 36,
          width: 32,
          height: 32,
          borderTop: "2px solid rgba(0, 240, 255, 0.7)",
          borderLeft: "2px solid rgba(0, 240, 255, 0.7)",
        }}
      />
      {/* Top Right */}
      <div
        style={{
          position: "absolute",
          top: 36,
          right: 36,
          width: 32,
          height: 32,
          borderTop: "2px solid rgba(0, 240, 255, 0.7)",
          borderRight: "2px solid rgba(0, 240, 255, 0.7)",
        }}
      />
      {/* Bottom Left */}
      <div
        style={{
          position: "absolute",
          bottom: 36,
          left: 36,
          width: 32,
          height: 32,
          borderBottom: "2px solid rgba(0, 240, 255, 0.7)",
          borderLeft: "2px solid rgba(0, 240, 255, 0.7)",
        }}
      />
      {/* Bottom Right */}
      <div
        style={{
          position: "absolute",
          bottom: 36,
          right: 36,
          width: 32,
          height: 32,
          borderBottom: "2px solid rgba(0, 240, 255, 0.7)",
          borderRight: "2px solid rgba(0, 240, 255, 0.7)",
        }}
      />

      {/* TOP STATUS BAR */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          fontSize: 14,
          letterSpacing: "0.15em",
          textTransform: "uppercase",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              backgroundColor: "#10B981",
              boxShadow: "0 0 10px #10B981",
            }}
          />
          <span style={{ color: "#00F0FF", fontWeight: 700 }}>
            {statusText}
          </span>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <span style={{ color: "#E2E8F0" }}>{fpsText}</span>
          <span style={{ color: "#94A3B8" }}>{timecode}</span>
        </div>
      </div>

      {/* BOTTOM STATUS BAR */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "flex-end",
          fontSize: 13,
          letterSpacing: "0.12em",
        }}
      >
        <div>
          <div style={{ color: "rgba(255,255,255,0.4)", marginBottom: 4 }}>
            INDEX // {sceneIndex}
          </div>
          <div
            style={{
              color: "#FFFFFF",
              fontWeight: 800,
              fontSize: 16,
              letterSpacing: "0.08em",
            }}
          >
            {sceneTitle}
          </div>
        </div>

        {/* EQUALIZER BARS */}
        <div style={{ display: "flex", alignItems: "flex-end", gap: 4 }}>
          {Array.from({ length: 8 }).map((_, i) => {
            const h = 8 + Math.abs(Math.sin(frame * 0.2 + i * 0.8)) * 26;
            return (
              <div
                key={i}
                style={{
                  width: 4,
                  height: h,
                  borderRadius: 2,
                  backgroundColor: i > 5 ? "#F97316" : "#00F0FF",
                  boxShadow: "0 0 6px rgba(0, 240, 255, 0.5)",
                }}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
};
