import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { AgentCursor } from "../components/AgentCursor";
import { THEME } from "../theme";

export const Scene4Browser: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const browserSpring = spring({
    frame,
    fps,
    config: { damping: 16, stiffness: 100 },
  });

  // Agent cursor moving across browser tab
  const cursorX = interpolate(frame, [0, 100, 220, 360], [1200, 1920, 2800, 2800], { extrapolateRight: "clamp" });
  const cursorY = interpolate(frame, [0, 100, 220, 360], [1400, 850, 1150, 1150], { extrapolateRight: "clamp" });

  const starCount = frame > 240 ? "⭐ 1,420" : "⭐ 1,419";

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
      {/* Header Title */}
      <div
        style={{
          position: "absolute",
          top: 120,
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
            padding: "8px 28px",
            borderRadius: 999,
            border: "1px solid rgba(47, 107, 255, 0.4)",
          }}
        >
          DIRECT CHROME NATIVE MESSAGING
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
          YOUR REAL BROWSER. <span style={{ color: THEME.cyan }}>YOUR LOGINS INTACT.</span>
        </h2>
      </div>

      {/* Browser Mockup Window */}
      <div
        style={{
          width: 3200,
          height: 1450,
          marginTop: 200,
          background: "rgba(15, 23, 42, 0.95)",
          border: `2px solid rgba(47, 107, 255, 0.5)`,
          borderRadius: 32,
          boxShadow: `0 30px 90px rgba(0, 0, 0, 0.8), 0 0 50px ${THEME.blueGlow}`,
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
          position: "relative",
          transform: `scale(${browserSpring})`,
          opacity: browserSpring,
        }}
      >
        {/* Browser Tab Bar */}
        <div
          style={{
            height: 80,
            background: "#0B0F19",
            borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
            display: "flex",
            alignItems: "center",
            padding: "0 28px",
            gap: 20,
          }}
        >
          {/* Window Control Buttons */}
          <div style={{ display: "flex", gap: 12, marginRight: 24 }}>
            <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#EF4444" }} />
            <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#F59E0B" }} />
            <div style={{ width: 18, height: 18, borderRadius: "50%", background: "#10B981" }} />
          </div>

          {/* Active fastuse Tab */}
          <div
            style={{
              background: "#1E293B",
              borderTop: `3px solid ${THEME.blue}`,
              borderRadius: "14px 14px 0 0",
              padding: "16px 36px",
              display: "flex",
              alignItems: "center",
              gap: 16,
            }}
          >
            <div style={{ width: 14, height: 14, borderRadius: "50%", background: THEME.blue }} />
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 24, fontWeight: 700, color: THEME.text }}>
              fastuse Tab Group — GitHub
            </span>
          </div>

          {/* Other Tabs (Your Personal Logged-In Tabs) */}
          <div
            style={{
              padding: "16px 32px",
              display: "flex",
              alignItems: "center",
              gap: 16,
              opacity: 0.6,
            }}
          >
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 22, color: THEME.textMuted }}>
              Gmail (Logged In)
            </span>
          </div>
          <div
            style={{
              padding: "16px 32px",
              display: "flex",
              alignItems: "center",
              gap: 16,
              opacity: 0.6,
            }}
          >
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 22, color: THEME.textMuted }}>
              Stripe Dashboard (Safe)
            </span>
          </div>
        </div>

        {/* Address Bar Row */}
        <div
          style={{
            height: 74,
            background: "#131B2E",
            borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
            display: "flex",
            alignItems: "center",
            padding: "0 36px",
            gap: 24,
          }}
        >
          <div
            style={{
              flex: 1,
              background: "rgba(255, 255, 255, 0.06)",
              borderRadius: 999,
              height: 48,
              display: "flex",
              alignItems: "center",
              padding: "0 28px",
              gap: 16,
            }}
          >
            <span style={{ color: THEME.emerald, fontSize: 20 }}>🔒</span>
            <span style={{ fontFamily: THEME.monoFamily, fontSize: 22, color: THEME.text }}>
              https://github.com/anassagd432/fastuse
            </span>
          </div>
        </div>

        {/* Browser Page Body */}
        <div
          style={{
            flex: 1,
            position: "relative",
            background: "#0D1117",
            padding: "60px 80px",
            display: "flex",
            flexDirection: "column",
            gap: 40,
          }}
        >
          {/* In-Page Shadow DOM Fastuse Banner */}
          <div
            style={{
              alignSelf: "center",
              background: "rgba(14, 19, 31, 0.96)",
              border: `2px solid ${THEME.blue}`,
              borderRadius: 999,
              padding: "12px 36px",
              display: "flex",
              alignItems: "center",
              gap: 16,
              boxShadow: `0 0 24px ${THEME.blueGlow}`,
            }}
          >
            <div style={{ width: 14, height: 14, borderRadius: "50%", background: THEME.blue }} />
            <span style={{ fontFamily: THEME.fontFamily, fontSize: 22, fontWeight: 700, color: THEME.text }}>
              This browser tab is being controlled by fastuse
            </span>
          </div>

          {/* GitHub Header Mockup */}
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              borderBottom: "1px solid #30363D",
              paddingBottom: 32,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 44, fontWeight: 800, color: THEME.text }}>
                anassagd432 / <span style={{ color: THEME.blueLight }}>fastuse</span>
              </span>
              <span
                style={{
                  fontFamily: THEME.monoFamily,
                  fontSize: 20,
                  color: THEME.textMuted,
                  border: "1px solid #30363D",
                  padding: "4px 14px",
                  borderRadius: 999,
                }}
              >
                Public
              </span>
            </div>

            {/* Star Button */}
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 14,
                background: "#21262D",
                border: "1px solid #30363D",
                borderRadius: 12,
                padding: "16px 36px",
                fontFamily: THEME.fontFamily,
                fontSize: 28,
                fontWeight: 700,
                color: THEME.text,
                boxShadow: frame > 240 ? `0 0 30px ${THEME.cyan}` : "none",
              }}
            >
              {starCount}
            </div>
          </div>

          {/* Feature Highlights Pills */}
          <div style={{ display: "flex", gap: 32, marginTop: 40 }}>
            <div
              style={{
                background: "rgba(16, 185, 129, 0.12)",
                border: "1px solid rgba(16, 185, 129, 0.3)",
                padding: "20px 40px",
                borderRadius: 20,
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 28, fontWeight: 800, color: THEME.emerald }}>
                ✓ Existing Cookies Preserved
              </span>
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 20, color: THEME.textMuted }}>
                No annoying re-logins or bot Captcha walls
              </span>
            </div>

            <div
              style={{
                background: "rgba(56, 189, 248, 0.12)",
                border: "1px solid rgba(56, 189, 248, 0.3)",
                padding: "20px 40px",
                borderRadius: 20,
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 28, fontWeight: 800, color: THEME.cyan }}>
                ✓ Instant DOM Tree Parsing
              </span>
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 20, color: THEME.textMuted }}>
                Compact sub-2ms accessibility snapshots
              </span>
            </div>

            <div
              style={{
                background: "rgba(47, 107, 255, 0.12)",
                border: "1px solid rgba(47, 107, 255, 0.3)",
                padding: "20px 40px",
                borderRadius: 20,
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 28, fontWeight: 800, color: THEME.blueLight }}>
                ✓ Multi-Step Auto-Retry
              </span>
              <span style={{ fontFamily: THEME.fontFamily, fontSize: 20, color: THEME.textMuted }}>
                Dropdowns and modals settle automatically
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Agent Cursor with Click Ripple on Star Button */}
      <AgentCursor x={cursorX} y={cursorY} clickFrame={240} scale={2.4} />
    </div>
  );
};
