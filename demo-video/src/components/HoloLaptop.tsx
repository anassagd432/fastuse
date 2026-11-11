import React from "react";
import { useCurrentFrame, spring, useVideoConfig } from "remotion";

interface HoloLaptopProps {
  children?: React.ReactNode;
  subtitle?: string;
  badgeText?: string;
}

export const HoloLaptop: React.FC<HoloLaptopProps> = ({
  children,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Floating gentle hover breathing motion
  const hoverY = Math.sin(frame * 0.04) * 6;
  const hoverRotateX = 24 + Math.cos(frame * 0.03) * 2;
  const hoverRotateY = Math.sin(frame * 0.035) * 2.5;

  // Intro entrance scale spring
  const introScale = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 90, mass: 0.8 },
  });

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        perspective: 1400,
        transformStyle: "preserve-3d",
      }}
    >
      {/* 3D LAPTOP WRAPPER */}
      <div
        style={{
          transform: `scale(${introScale}) translateY(${hoverY}px) rotateX(${hoverRotateX}deg) rotateY(${hoverRotateY}deg)`,
          transformStyle: "preserve-3d",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
        }}
      >
        {/* LAPTOP SCREEN (TOP LID) */}
        <div
          style={{
            width: 780,
            height: 480,
            borderRadius: "18px 18px 4px 4px",
            backgroundColor: "#0A0D14",
            border: "3px solid #1E293B",
            boxShadow:
              "0 0 50px rgba(239, 68, 68, 0.25), inset 0 0 20px rgba(255, 255, 255, 0.05), 0 30px 60px rgba(0,0,0,0.8)",
            padding: 12,
            position: "relative",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          {/* Webcam dot */}
          <div
            style={{
              position: "absolute",
              top: 5,
              left: "50%",
              transform: "translateX(-50%)",
              width: 5,
              height: 5,
              borderRadius: "50%",
              backgroundColor: "#1E293B",
              border: "1px solid #334155",
            }}
          />

          {/* INNER DISPLAY */}
          <div
            style={{
              flex: 1,
              borderRadius: 8,
              backgroundColor: "#030712",
              overflow: "hidden",
              position: "relative",
              border: "1px solid #111827",
            }}
          >
            {children}

            {/* Screen Glass Reflection Shimmer */}
            <div
              style={{
                position: "absolute",
                top: 0,
                left: "-50%",
                width: "200%",
                height: "100%",
                background:
                  "linear-gradient(115deg, transparent 40%, rgba(255,255,255,0.06) 50%, transparent 60%)",
                transform: `translateX(${(frame * 1.5) % 300}px)`,
                pointerEvents: "none",
              }}
            />
          </div>
        </div>

        {/* LAPTOP HINGE */}
        <div
          style={{
            width: 220,
            height: 8,
            backgroundColor: "#1E293B",
            borderRadius: "0 0 4px 4px",
          }}
        />

        {/* LAPTOP BASE (KEYBOARD CHASSIS) */}
        <div
          style={{
            width: 820,
            height: 180,
            backgroundColor: "#0D1117",
            borderRadius: "4px 4px 20px 20px",
            border: "2px solid #1F2937",
            boxShadow:
              "0 20px 40px rgba(0,0,0,0.9), inset 0 1px 2px rgba(255,255,255,0.15)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            paddingTop: 14,
            transform: "rotateX(62deg)",
            transformOrigin: "top center",
          }}
        >
          {/* Keyboard Well */}
          <div
            style={{
              width: 680,
              height: 90,
              backgroundColor: "#080C14",
              borderRadius: 6,
              border: "1px solid #1E293B",
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-around",
              padding: "4px 8px",
            }}
          >
            {[0, 1, 2].map((r) => (
              <div
                key={r}
                style={{
                  display: "flex",
                  gap: 4,
                  height: 22,
                  justifyContent: "center",
                }}
              >
                {Array.from({ length: 14 }).map((_, k) => (
                  <div
                    key={k}
                    style={{
                      flex: 1,
                      backgroundColor: "#111827",
                      borderRadius: 3,
                      border: "1px solid #1F2937",
                      boxShadow: "0 1px 2px rgba(0,0,0,0.5)",
                    }}
                  />
                ))}
              </div>
            ))}
          </div>

          {/* Trackpad */}
          <div
            style={{
              marginTop: 10,
              width: 140,
              height: 48,
              borderRadius: 4,
              border: "1px solid #1E293B",
              backgroundColor: "#0B0F19",
            }}
          />
        </div>

        {/* MIRROR DESK REFLECTION */}
        <div
          style={{
            width: 780,
            height: 260,
            marginTop: -50,
            transform: "scaleY(-0.8) perspective(1200px) rotateX(25deg)",
            opacity: 0.35,
            filter: "blur(5px)",
            maskImage:
              "linear-gradient(to bottom, rgba(0,0,0,1) 0%, rgba(0,0,0,0.2) 50%, transparent 80%)",
            WebkitMaskImage:
              "linear-gradient(to bottom, rgba(0,0,0,1) 0%, rgba(0,0,0,0.2) 50%, transparent 80%)",
            pointerEvents: "none",
            borderRadius: "18px",
            overflow: "hidden",
            backgroundColor: "#0A0D14",
            border: "2px solid rgba(239,68,68,0.2)",
          }}
        >
          {children}
        </div>
      </div>
    </div>
  );
};
