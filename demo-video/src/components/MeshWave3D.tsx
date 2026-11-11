import React, { useMemo } from "react";
import { useCurrentFrame, interpolate } from "remotion";

interface MeshWave3DProps {
  cols?: number;
  rows?: number;
  waveSpeed?: number;
  colorScheme?: "cyber" | "claude" | "neon";
}

export const MeshWave3D: React.FC<MeshWave3DProps> = ({
  cols = 24,
  rows = 24,
  waveSpeed = 1.0,
  colorScheme = "cyber",
}) => {
  const frame = useCurrentFrame();

  // 3D Perspective Projection Matrix
  // Center is (540, 960)
  const projectedPoints = useMemo(() => {
    const pts: { x: number; y: number; z: number; height: number }[][] = [];

    const cellW = 38;
    const cellD = 38;
    const originX = 540;
    const originY = 980;
    const tilt = 0.85; // isometric pitch

    for (let r = 0; r <= rows; r++) {
      const rowPts = [];
      for (let c = 0; c <= cols; c++) {
        // centered coordinates
        const gridX = (c - cols / 2) * cellW;
        const gridZ = (r - rows / 2) * cellD;

        // Radial distance from center
        const distFromCenter = Math.sqrt(gridX * gridX + gridZ * gridZ);

        // Undulating wave formula (ripples + longitudinal wave)
        const ripple = Math.sin(distFromCenter * 0.015 - frame * 0.08 * waveSpeed);
        const cross = Math.cos(gridX * 0.012 + frame * 0.05 * waveSpeed);
        const height = (ripple * 0.7 + cross * 0.3) * 65 * (1 - distFromCenter / 700);

        // 3D Isometric / perspective projection
        // Screen X = originX + (gridX - gridZ) * cos(30)
        // Screen Y = originY + (gridX + gridZ) * sin(30) * tilt - height
        const screenX = originX + (gridX - gridZ) * 0.866;
        const screenY = originY + (gridX + gridZ) * 0.5 * tilt - height;

        rowPts.push({ x: screenX, y: screenY, z: gridZ, height });
      }
      pts.push(rowPts);
    }
    return pts;
  }, [cols, rows, frame, waveSpeed]);

  const primaryColor =
    colorScheme === "claude" ? "#D97706" : colorScheme === "neon" ? "#A855F7" : "#00F0FF";
  const secondaryColor =
    colorScheme === "claude" ? "#F97316" : colorScheme === "neon" ? "#EC4899" : "#3B82F6";

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
          <linearGradient id="meshGradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor={primaryColor} stopOpacity="0.8" />
            <stop offset="50%" stopColor={secondaryColor} stopOpacity="0.6" />
            <stop offset="100%" stopColor="#8A2BE2" stopOpacity="0.3" />
          </linearGradient>

          <filter id="meshGlow" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Grid lines along rows */}
        {projectedPoints.map((row, rIdx) => {
          if (rIdx === 0) return null;
          let pathD = "";
          row.forEach((pt, cIdx) => {
            if (cIdx === 0) pathD += `M ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
            else pathD += ` L ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
          });
          const opacity = interpolate(rIdx, [0, rows / 2, rows], [0.3, 0.75, 0.2]);
          return (
            <path
              key={`row-${rIdx}`}
              d={pathD}
              fill="none"
              stroke="url(#meshGradient)"
              strokeWidth="1.2"
              opacity={opacity}
              filter="url(#meshGlow)"
            />
          );
        })}

        {/* Grid lines along columns */}
        {Array.from({ length: cols + 1 }).map((_, cIdx) => {
          let pathD = "";
          for (let rIdx = 0; rIdx <= rows; rIdx++) {
            const pt = projectedPoints[rIdx][cIdx];
            if (rIdx === 0) pathD += `M ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
            else pathD += ` L ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
          }
          const opacity = interpolate(cIdx, [0, cols / 2, cols], [0.3, 0.75, 0.2]);
          return (
            <path
              key={`col-${cIdx}`}
              d={pathD}
              fill="none"
              stroke="url(#meshGradient)"
              strokeWidth="1.2"
              opacity={opacity}
              filter="url(#meshGlow)"
            />
          );
        })}

        {/* Highlight vertex dots on high crests */}
        {projectedPoints.map((row, rIdx) =>
          row.map((pt, cIdx) => {
            if (pt.height > 25 && (rIdx + cIdx) % 2 === 0) {
              const radius = interpolate(pt.height, [25, 65], [1.5, 4.0]);
              return (
                <circle
                  key={`dot-${rIdx}-${cIdx}`}
                  cx={pt.x}
                  cy={pt.y}
                  r={radius}
                  fill="#FFFFFF"
                  opacity={0.85}
                  filter="url(#meshGlow)"
                />
              );
            }
            return null;
          })
        )}

        {/* Central glowing core energy pulse */}
        <circle
          cx="540"
          cy="980"
          r={28 + Math.sin(frame * 0.1) * 8}
          fill="none"
          stroke={primaryColor}
          strokeWidth="3"
          opacity={0.8}
          filter="url(#meshGlow)"
        />
        <circle cx="540" cy="980" r="10" fill="#FFFFFF" opacity={0.9} />
      </svg>
    </div>
  );
};
