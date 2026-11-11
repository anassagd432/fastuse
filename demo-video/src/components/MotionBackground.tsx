import React from "react";
import { interpolate, useCurrentFrame } from "remotion";
import { THEME } from "../theme";

interface MotionBackgroundProps {
  accentColor?: string;
  gridSpeed?: number;
}

export const MotionBackground: React.FC<MotionBackgroundProps> = ({
  accentColor = THEME.blue,
  gridSpeed = 1.0,
}) => {
  const frame = useCurrentFrame();

  // Floating ambient glow orbs
  const orb1X = interpolate(
    Math.sin((frame / 90) * Math.PI * 2),
    [-1, 1],
    [300, 1500]
  );
  const orb1Y = interpolate(
    Math.cos((frame / 120) * Math.PI * 2),
    [-1, 1],
    [600, 1800]
  );

  const orb2X = interpolate(
    Math.cos((frame / 100) * Math.PI * 2),
    [-1, 1],
    [1600, 500]
  );
  const orb2Y = interpolate(
    Math.sin((frame / 140) * Math.PI * 2),
    [-1, 1],
    [2400, 3200]
  );

  // Moving cyber grid floor
  const gridOffsetY = (frame * 3 * gridSpeed) % 120;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: THEME.bg,
        overflow: "hidden",
        pointerEvents: "none",
        zIndex: 0,
      }}
    >
      {/* Dynamic Ambient Gradient Orbs */}
      <div
        style={{
          position: "absolute",
          left: orb1X,
          top: orb1Y,
          width: 1400,
          height: 1400,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${accentColor} 0%, transparent 65%)`,
          opacity: 0.18,
          filter: "blur(120px)",
        }}
      />
      <div
        style={{
          position: "absolute",
          left: orb2X,
          top: orb2Y,
          width: 1600,
          height: 1600,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${THEME.cyan} 0%, transparent 65%)`,
          opacity: 0.12,
          filter: "blur(140px)",
        }}
      />

      {/* Cyber Grid with 3D Depth */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `
            linear-gradient(to right, rgba(255, 255, 255, 0.03) 2px, transparent 2px),
            linear-gradient(to bottom, rgba(255, 255, 255, 0.03) 2px, transparent 2px)
          `,
          backgroundSize: "120px 120px",
          backgroundPosition: `0px ${gridOffsetY}px`,
          opacity: 0.7,
        }}
      />

      {/* Vignette & Scanline Subtle Texture */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: "radial-gradient(circle at center, transparent 40%, rgba(7, 9, 14, 0.85) 100%)",
        }}
      />

      {/* Top and Bottom Screen Edge Lasers */}
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          height: 8,
          background: `linear-gradient(90deg, transparent, ${accentColor}, transparent)`,
          boxShadow: `0 0 30px ${accentColor}`,
        }}
      />
      <div
        style={{
          position: "absolute",
          bottom: 0,
          left: 0,
          right: 0,
          height: 8,
          background: `linear-gradient(90deg, transparent, ${THEME.cyan}, transparent)`,
          boxShadow: `0 0 30px ${THEME.cyan}`,
        }}
      />
    </div>
  );
};
