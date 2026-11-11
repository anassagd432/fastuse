import React from "react";
import { useCurrentFrame, spring, useVideoConfig } from "remotion";

interface StarburstBadgeProps {
  text: string;
  subtext?: string;
  color?: string;
  textColor?: string;
  size?: number;
  rotationOffset?: number;
}

export const StarburstBadge: React.FC<StarburstBadgeProps> = ({
  text,
  subtext,
  color = "#F97316", // Vibrant coral / orange like Claude starburst
  textColor = "#FFFFFF",
  size = 200,
  rotationOffset = -8,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Entrance spring
  const scale = spring({
    frame,
    fps,
    config: { damping: 10, stiffness: 120, mass: 0.6 },
  });

  // Floating gentle wobble
  const wobbleAngle = Math.sin(frame * 0.06) * 4 + rotationOffset;
  const floatY = Math.cos(frame * 0.05) * 6;

  // Generate 16-point starburst SVG path
  const points = 16;
  const outerR = size * 0.48;
  const innerR = size * 0.38;
  const cx = size / 2;
  const cy = size / 2;

  let pathD = "";
  for (let i = 0; i < points * 2; i++) {
    const angle = (i * Math.PI) / points;
    const r = i % 2 === 0 ? outerR : innerR;
    const x = cx + r * Math.cos(angle);
    const y = cy + r * Math.sin(angle);
    if (i === 0) pathD += `M ${x.toFixed(1)} ${y.toFixed(1)}`;
    else pathD += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
  }
  pathD += " Z";

  return (
    <div
      style={{
        width: size,
        height: size,
        transform: `translateY(${floatY}px) rotate(${wobbleAngle}deg) scale(${scale})`,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        position: "relative",
        filter: "drop-shadow(0 14px 28px rgba(0,0,0,0.65)) drop-shadow(0 0 20px rgba(249, 115, 22, 0.45))",
      }}
    >
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${size} ${size}`}
        style={{ position: "absolute", inset: 0 }}
      >
        <path d={pathD} fill={color} />
      </svg>

      <div
        style={{
          position: "relative",
          zIndex: 2,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          textAlign: "center",
          padding: "0 18px",
          fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
        }}
      >
        <span
          style={{
            fontSize: size * 0.13,
            fontWeight: 900,
            color: textColor,
            lineHeight: 1.05,
            letterSpacing: "-0.03em",
            textTransform: "uppercase",
            textShadow: "0 2px 4px rgba(0,0,0,0.5)",
          }}
        >
          {text}
        </span>
        {subtext && (
          <span
            style={{
              fontSize: size * 0.08,
              fontWeight: 700,
              color: "rgba(255,255,255,0.9)",
              marginTop: 2,
              letterSpacing: "0.05em",
              textTransform: "uppercase",
            }}
          >
            {subtext}
          </span>
        )}
      </div>
    </div>
  );
};
