import React, { useMemo } from "react";
import { useCurrentFrame, interpolate } from "remotion";

interface ParticleVortexProps {
  particleCount?: number;
  coreColor?: string;
}

export const ParticleVortex: React.FC<ParticleVortexProps> = ({
  particleCount = 180,
  coreColor = "#00F0FF",
}) => {
  const frame = useCurrentFrame();

  const particles = useMemo(() => {
    const list = [];
    for (let i = 0; i < particleCount; i++) {
      const arm = i % 4; // 4 spiral arms
      const baseAngle = (arm * Math.PI) / 2 + (i / particleCount) * Math.PI * 4;
      const baseRadius = 40 + Math.pow(i / particleCount, 1.4) * 600;
      const speed = 0.8 + ((i * 17) % 50) / 40;
      const size = 1.2 + (i % 4) * 0.8;
      const color =
        i % 5 === 0
          ? "#FFFFFF"
          : i % 5 === 1
          ? "#00F0FF"
          : i % 5 === 2
          ? "#8A2BE2"
          : i % 5 === 3
          ? "#38BDF8"
          : "#F59E0B";
      list.push({ baseAngle, baseRadius, speed, size, color });
    }
    return list;
  }, [particleCount]);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        backgroundColor: "transparent",
      }}
    >
      <svg
        width="100%"
        height="100%"
        viewBox="0 0 1080 1920"
        style={{ position: "absolute", inset: 0 }}
      >
        <defs>
          <filter id="vortexGlow">
            <feGaussianBlur stdDeviation="2.5" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {particles.map((p, idx) => {
          // Inward swirling vortex physics
          const curAngle = p.baseAngle + frame * 0.04 * p.speed;
          // Radius oscillates or breathes
          const spiralR = p.baseRadius * (1 + Math.sin(frame * 0.03 + idx * 0.1) * 0.08);

          const cx = 540;
          const cy = 960;

          // Head position
          const x = cx + Math.cos(curAngle) * spiralR;
          const y = cy + Math.sin(curAngle) * spiralR;

          // Tail position for motion streak
          const tailAngle = curAngle - 0.12 * p.speed;
          const tailR = spiralR * 1.03;
          const xTail = cx + Math.cos(tailAngle) * tailR;
          const yTail = cy + Math.sin(tailAngle) * tailR;

          const opacity = interpolate(
            spiralR,
            [40, 180, 500, 650],
            [0.2, 0.95, 0.8, 0],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
          );

          return (
            <g key={idx}>
              <line
                x1={xTail}
                y1={yTail}
                x2={x}
                y2={y}
                stroke={p.color}
                strokeWidth={p.size}
                strokeLinecap="round"
                opacity={opacity}
                filter="url(#vortexGlow)"
              />
              <circle
                cx={x}
                cy={y}
                r={p.size * 0.9}
                fill="#FFFFFF"
                opacity={opacity * 0.95}
              />
            </g>
          );
        })}

        {/* Pulsing singularity core */}
        <circle
          cx="540"
          cy="960"
          r={24 + Math.sin(frame * 0.12) * 6}
          fill="none"
          stroke={coreColor}
          strokeWidth="3.5"
          opacity={0.85}
          filter="url(#vortexGlow)"
        />
        <circle cx="540" cy="960" r="8" fill="#FFFFFF" opacity={0.95} />
      </svg>
    </div>
  );
};
