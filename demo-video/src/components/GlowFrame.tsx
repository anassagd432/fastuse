import React from "react";
import { interpolate, useCurrentFrame } from "remotion";
import { THEME } from "../theme";

interface GlowFrameProps {
  opacity?: number;
  bannerText?: string;
}

export const GlowFrame: React.FC<GlowFrameProps> = ({
  opacity = 1.0,
  bannerText = "This computer is being controlled by fastuse",
}) => {
  const frame = useCurrentFrame();

  const pulse = interpolate(
    Math.sin((frame / 60) * Math.PI * 2),
    [-1, 1],
    [0.85, 1.0]
  );

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        zIndex: 9000,
        opacity,
      }}
    >
      {/* Screen Edge Aura */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          boxShadow: `inset 0 0 80px 10px rgba(47, 107, 255, ${0.45 * pulse})`,
          border: `2px solid rgba(47, 107, 255, ${0.7 * pulse})`,
        }}
      />

      {/* Top Banner Pill */}
      {bannerText && (
        <div
          style={{
            position: "absolute",
            top: 40,
            left: "50%",
            transform: "translateX(-50%)",
            background: "rgba(14, 19, 31, 0.92)",
            backdropFilter: "blur(16px)",
            border: "1.5px solid rgba(47, 107, 255, 0.6)",
            boxShadow: "0 8px 30px rgba(0, 0, 0, 0.5), 0 0 24px rgba(47, 107, 255, 0.35)",
            borderRadius: 999,
            padding: "14px 36px",
            display: "flex",
            alignItems: "center",
            gap: 16,
          }}
        >
          {/* Pulsing Status Dot */}
          <div
            style={{
              width: 14,
              height: 14,
              borderRadius: "50%",
              background: THEME.blue,
              boxShadow: `0 0 12px ${THEME.blue}`,
              transform: `scale(${pulse})`,
            }}
          />
          <span
            style={{
              color: THEME.text,
              fontFamily: THEME.fontFamily,
              fontSize: 24,
              fontWeight: 600,
              letterSpacing: 0.5,
            }}
          >
            {bannerText}
          </span>
        </div>
      )}
    </div>
  );
};
