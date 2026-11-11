import React from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { THEME } from "../theme";

interface Metric {
  label: string;
  legacyVal: string;
  fastVal: string;
  speedup: string;
  legacyMs: number;
  fastMs: number;
}

const METRICS: Metric[] = [
  {
    label: "List Window Enumeration",
    legacyVal: "437 ms",
    fastVal: "0.9 ms",
    speedup: "485x FASTER",
    legacyMs: 437,
    fastMs: 0.9,
  },
  {
    label: "Direct Element Click",
    legacyVal: "16.0 ms",
    fastVal: "1.8 ms",
    speedup: "9x FASTER",
    legacyMs: 16,
    fastMs: 1.8,
  },
  {
    label: "UI Tree Cache & Walk",
    legacyVal: "160 ms",
    fastVal: "29 ms",
    speedup: "5.5x FASTER",
    legacyMs: 160,
    fastMs: 29,
  },
];

export const BenchmarkCard: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const cardSpring = spring({
    frame,
    fps,
    config: { damping: 18, stiffness: 120 },
  });

  return (
    <div
      style={{
        width: 1700,
        background: "rgba(14, 19, 31, 0.95)",
        backdropFilter: "blur(24px)",
        border: `2px solid ${THEME.bgCardBorder}`,
        borderRadius: 32,
        padding: "60px 80px",
        boxShadow: "0 24px 80px rgba(0, 0, 0, 0.8), 0 0 40px rgba(47, 107, 255, 0.2)",
        transform: `scale(${cardSpring})`,
        opacity: cardSpring,
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 48,
          borderBottom: `2px solid ${THEME.bgCardBorder}`,
          paddingBottom: 28,
        }}
      >
        <span
          style={{
            fontFamily: THEME.fontFamily,
            fontSize: 36,
            fontWeight: 800,
            color: THEME.text,
            letterSpacing: 1,
            textTransform: "uppercase",
          }}
        >
          ⚡ Latency Benchmark (Persistent MCP Session)
        </span>
        <span
          style={{
            fontFamily: THEME.monoFamily,
            fontSize: 26,
            color: THEME.cyan,
            background: "rgba(56, 189, 248, 0.12)",
            padding: "8px 20px",
            borderRadius: 999,
            border: "1px solid rgba(56, 189, 248, 0.3)",
          }}
        >
          Measured on Win 11 / macOS
        </span>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 36 }}>
        {METRICS.map((m, idx) => {
          const rowDelay = idx * 12;
          const rowProgress = spring({
            frame: frame - rowDelay,
            fps,
            config: { damping: 16, stiffness: 100 },
          });

          const barWidth = interpolate(rowProgress, [0, 1], [0, 100]);

          return (
            <div
              key={m.label}
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 14,
                opacity: rowProgress,
                transform: `translateX(${interpolate(rowProgress, [0, 1], [-40, 0])}px)`,
              }}
            >
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <span
                  style={{
                    fontFamily: THEME.fontFamily,
                    fontSize: 32,
                    fontWeight: 700,
                    color: THEME.text,
                  }}
                >
                  {m.label}
                </span>

                <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
                  <span
                    style={{
                      fontFamily: THEME.monoFamily,
                      fontSize: 28,
                      color: THEME.red,
                      textDecoration: "line-through",
                      opacity: 0.8,
                    }}
                  >
                    Legacy: {m.legacyVal}
                  </span>
                  <span
                    style={{
                      fontFamily: THEME.monoFamily,
                      fontSize: 36,
                      fontWeight: 800,
                      color: THEME.cyan,
                    }}
                  >
                    fastuse: {m.fastVal}
                  </span>
                  <span
                    style={{
                      fontFamily: THEME.monoFamily,
                      fontSize: 24,
                      fontWeight: 800,
                      color: "#059669",
                      background: "rgba(16, 185, 129, 0.15)",
                      border: "1px solid rgba(16, 185, 129, 0.4)",
                      padding: "6px 18px",
                      borderRadius: 12,
                    }}
                  >
                    {m.speedup}
                  </span>
                </div>
              </div>

              {/* Progress Bar Container */}
              <div
                style={{
                  width: "100%",
                  height: 18,
                  background: "rgba(255, 255, 255, 0.06)",
                  borderRadius: 999,
                  overflow: "hidden",
                  position: "relative",
                }}
              >
                <div
                  style={{
                    width: `${barWidth}%`,
                    height: "100%",
                    background: `linear-gradient(90deg, ${THEME.blue}, ${THEME.cyan})`,
                    boxShadow: `0 0 16px ${THEME.blue}`,
                    borderRadius: 999,
                  }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
