import React from "react";
import { interpolate, useCurrentFrame } from "remotion";
import { THEME } from "../theme";

interface AgentCursorProps {
  x: number;
  y: number;
  clickFrame?: number; // frame at which a click ripple occurs
  scale?: number;
}

export const AgentCursor: React.FC<AgentCursorProps> = ({
  x,
  y,
  clickFrame,
  scale = 1.0,
}) => {
  const frame = useCurrentFrame();

  const isClicking = clickFrame !== undefined && frame >= clickFrame;
  const clickProgress = isClicking
    ? interpolate(frame - clickFrame, [0, 20], [0, 1], {
        extrapolateRight: "clamp",
      })
    : 0;

  const rippleRadius = interpolate(clickProgress, [0, 1], [0, 70]);
  const rippleOpacity = interpolate(clickProgress, [0, 0.7, 1], [0.9, 0.4, 0]);

  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        pointerEvents: "none",
        zIndex: 9999,
        transform: `scale(${scale})`,
        transformOrigin: "top left",
      }}
    >
      {/* Click Ripple Effect */}
      {isClicking && clickProgress < 1 && (
        <div
          style={{
            position: "absolute",
            left: -rippleRadius,
            top: -rippleRadius,
            width: rippleRadius * 2,
            height: rippleRadius * 2,
            borderRadius: "50%",
            border: `3px solid ${THEME.cyan}`,
            boxShadow: `0 0 20px ${THEME.blue}`,
            opacity: rippleOpacity,
            pointerEvents: "none",
          }}
        />
      )}

      {/* SVG Blue Agent Pointer */}
      <svg
        width="38"
        height="44"
        viewBox="0 0 38 44"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{
          filter: "drop-shadow(0 6px 14px rgba(0, 0, 0, 0.6)) drop-shadow(0 0 12px rgba(47, 107, 255, 0.5))",
        }}
      >
        <path
          d="M3 2L15.5 38.5L21.5 24.5L35.5 20.5L3 2Z"
          fill={THEME.blue}
          stroke="#FFFFFF"
          strokeWidth="2.5"
          strokeLinejoin="round"
        />
      </svg>
    </div>
  );
};
