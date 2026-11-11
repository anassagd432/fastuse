import React, { useMemo } from "react";
import { useCurrentFrame, interpolate } from "remotion";

interface TunnelWarpProps {
  speed?: number;
  hueShift?: number;
  glowColor?: string;
  tunnelDepth?: number;
}

export const TunnelWarp: React.FC<TunnelWarpProps> = ({
  speed = 1.0,
  hueShift = 0,
  glowColor = "#00F0FF",
  tunnelDepth = 800,
}) => {
  const frame = useCurrentFrame();

  // Generate 140 deterministic particle rays
  const rays = useMemo(() => {
    const list = [];
    for (let i = 0; i < 140; i++) {
      const angle = (i / 140) * Math.PI * 2 + (i % 7) * 0.1;
      const baseDistance = 30 + ((i * 137.5) % 850);
      const speedMultiplier = 0.8 + ((i * 31) % 100) / 70;
      const size = 1.5 + (i % 3);
      const color =
        i % 4 === 0
          ? "#00F0FF" // cyan
          : i % 4 === 1
          ? "#8A2BE2" // purple
          : i % 4 === 2
          ? "#3B82F6" // blue
          : "#FFB800"; // gold highlight
      list.push({ angle, baseDistance, speedMultiplier, size, color });
    }
    return list;
  }, []);

  const rotation = frame * 0.35 * speed;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        backgroundColor: "#05070D",
        perspective: `${tunnelDepth}px`,
      }}
    >
      {/* Central hyperspace glow */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          width: 380,
          height: 380,
          transform: "translate(-50%, -50%)",
          borderRadius: "50%",
          background: `radial-gradient(circle, ${glowColor}66 0%, ${glowColor}11 50%, transparent 80%)`,
          filter: "blur(40px)",
          pointerEvents: "none",
        }}
      />

      {/* SVG Streaks */}
      <svg
        width="100%"
        height="100%"
        viewBox="0 0 1080 1920"
        style={{ position: "absolute", inset: 0 }}
      >
        <defs>
          <radialGradient id="centerCore" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#FFFFFF" stopOpacity="0.9" />
            <stop offset="40%" stopColor={glowColor} stopOpacity="0.6" />
            <stop offset="100%" stopColor="#000000" stopOpacity="0" />
          </radialGradient>
        </defs>

        <g transform={`rotate(${rotation} 540 960)`}>
          {rays.map((ray, idx) => {
            // Distance travels from center outward in loop
            const progress = ((frame * 2.8 * ray.speedMultiplier * speed + ray.baseDistance) % 1100) / 1100;
            const dist = Math.pow(progress, 1.8) * 1100;
            const tailDist = Math.max(0, dist - (40 + progress * 140));

            const xHead = 540 + Math.cos(ray.angle) * dist;
            const yHead = 960 + Math.sin(ray.angle) * dist;
            const xTail = 540 + Math.cos(ray.angle) * tailDist;
            const yTail = 960 + Math.sin(ray.angle) * tailDist;

            const opacity = interpolate(
              progress,
              [0, 0.15, 0.85, 1],
              [0, 0.95, 0.95, 0],
              { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
            );

            const strokeWidth = ray.size * (0.8 + progress * 2.2);

            return (
              <g key={idx}>
                <line
                  x1={xTail}
                  y1={yTail}
                  x2={xHead}
                  y2={yHead}
                  stroke={ray.color}
                  strokeWidth={strokeWidth}
                  strokeLinecap="round"
                  opacity={opacity}
                />
                {/* Glowing head spark */}
                <circle
                  cx={xHead}
                  cy={yHead}
                  r={strokeWidth * 0.8}
                  fill="#FFFFFF"
                  opacity={opacity * 0.9}
                />
              </g>
            );
          })}
        </g>

        {/* Concentric hyperspace shock rings */}
        {[0, 1, 2, 3].map((rIdx) => {
          const ringProgress = ((frame * 1.5 + rIdx * 25) % 100) / 100;
          const rRadius = Math.pow(ringProgress, 2) * 850;
          const rOpacity = interpolate(
            ringProgress,
            [0, 0.2, 0.8, 1],
            [0, 0.4, 0.2, 0]
          );
          return (
            <circle
              key={`ring-${rIdx}`}
              cx="540"
              cy="960"
              r={rRadius}
              fill="none"
              stroke={glowColor}
              strokeWidth={1.5 + (1 - ringProgress) * 3}
              opacity={rOpacity}
              strokeDasharray="8 12"
            />
          );
        })}

        {/* Center core light */}
        <circle cx="540" cy="960" r="18" fill="url(#centerCore)" />
      </svg>
    </div>
  );
};
