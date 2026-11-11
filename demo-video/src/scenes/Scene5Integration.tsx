import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";
import { THEME } from "../theme";

export const Scene5Integration: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const terminalSpring = spring({
    frame,
    fps,
    config: { damping: 16, stiffness: 100 },
  });

  const check1 = spring({ frame: frame - 40, fps, config: { damping: 14, stiffness: 120 } });
  const check2 = spring({ frame: frame - 80, fps, config: { damping: 14, stiffness: 120 } });
  const check3 = spring({ frame: frame - 120, fps, config: { damping: 14, stiffness: 120 } });
  const check4 = spring({ frame: frame - 160, fps, config: { damping: 14, stiffness: 120 } });

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
      {/* Title */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 16,
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
            padding: "8px 28px",
            borderRadius: 999,
            border: "1px solid rgba(47, 107, 255, 0.4)",
          }}
        >
          ZERO CONFIGURATION
        </span>
        <h2
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 88,
            fontWeight: 900,
            color: THEME.text,
            margin: 0,
            letterSpacing: -1,
          }}
        >
          ONE PROMPT INTO <span style={{ color: THEME.cyan }}>ANTIGRAVITY.</span>
        </h2>
      </div>

      {/* Antigravity Chat Mockup Container */}
      <div
        style={{
          width: 2600,
          background: "rgba(14, 19, 31, 0.95)",
          border: `2px solid ${THEME.bgCardBorder}`,
          borderRadius: 32,
          boxShadow: `0 30px 90px rgba(0, 0, 0, 0.8), 0 0 50px ${THEME.blueGlow}`,
          overflow: "hidden",
          padding: "60px 80px",
          display: "flex",
          flexDirection: "column",
          gap: 40,
          transform: `scale(${terminalSpring})`,
          opacity: terminalSpring,
        }}
      >
        {/* Antigravity Prompt Card */}
        <div
          style={{
            background: "rgba(47, 107, 255, 0.1)",
            border: "1.5px solid rgba(47, 107, 255, 0.3)",
            borderRadius: 20,
            padding: "32px 48px",
            display: "flex",
            alignItems: "center",
            gap: 28,
          }}
        >
          <div
            style={{
              width: 56,
              height: 56,
              borderRadius: "50%",
              background: `linear-gradient(135deg, ${THEME.blue}, ${THEME.cyan})`,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 28,
              fontWeight: 800,
              color: "#FFF",
            }}
          >
            A
          </div>
          <span
            style={{
              fontFamily: THEME.monoFamily,
              fontSize: 32,
              color: THEME.text,
              lineHeight: 1.4,
            }}
          >
            "Install and configure the <span style={{ color: THEME.cyan, fontWeight: 700 }}>fastuse</span> MCP server on this computer..."
          </span>
        </div>

        {/* Verification Checkmark Sequence */}
        <div style={{ display: "flex", flexDirection: "column", gap: 24, marginTop: 10 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 24,
              opacity: check1,
              transform: `translateX(${spring({ frame: frame - 40, fps, config: { damping: 14 } })}px)`,
            }}
          >
            <div
              style={{
                width: 44,
                height: 44,
                borderRadius: "50%",
                background: "rgba(16, 185, 129, 0.2)",
                border: "2px solid #10B981",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "#10B981",
                fontSize: 24,
                fontWeight: 800,
              }}
            >
              ✓
            </div>
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 32, fontWeight: 700, color: THEME.text }}>
              Native Release Binary Verified & Compiled (<span style={{ color: THEME.cyan }}>Rust</span>)
            </span>
          </div>

          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 24,
              opacity: check2,
              transform: `translateX(${spring({ frame: frame - 80, fps, config: { damping: 14 } })}px)`,
            }}
          >
            <div
              style={{
                width: 44,
                height: 44,
                borderRadius: "50%",
                background: "rgba(16, 185, 129, 0.2)",
                border: "2px solid #10B981",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "#10B981",
                fontSize: 24,
                fontWeight: 800,
              }}
            >
              ✓
            </div>
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 32, fontWeight: 700, color: THEME.text }}>
              Chrome Native Messaging Host Linked (<span style={{ color: THEME.cyan }}>Brave · Chrome · Edge</span>)
            </span>
          </div>

          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 24,
              opacity: check3,
              transform: `translateX(${spring({ frame: frame - 120, fps, config: { damping: 14 } })}px)`,
            }}
          >
            <div
              style={{
                width: 44,
                height: 44,
                borderRadius: "50%",
                background: "rgba(16, 185, 129, 0.2)",
                border: "2px solid #10B981",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "#10B981",
                fontSize: 24,
                fontWeight: 800,
              }}
            >
              ✓
            </div>
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 32, fontWeight: 700, color: THEME.text }}>
              Default <span style={{ color: THEME.cyan }}>GEMINI.md</span> Auto-Routing Rule Active
            </span>
          </div>

          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 24,
              opacity: check4,
              transform: `translateX(${spring({ frame: frame - 160, fps, config: { damping: 14 } })}px)`,
            }}
          >
            <div
              style={{
                width: 44,
                height: 44,
                borderRadius: "50%",
                background: "rgba(16, 185, 129, 0.2)",
                border: "2px solid #10B981",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "#10B981",
                fontSize: 24,
                fontWeight: 800,
              }}
            >
              ✓
            </div>
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 32, fontWeight: 700, color: THEME.emerald }}>
              `fastuse doctor` Completed: 100% Ready for Computer Use
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};
