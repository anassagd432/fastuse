import React from "react";
import { Audio, Sequence, staticFile } from "remotion";
import { VerticalScene1Hijack } from "./scenes/VerticalScene1Hijack";
import { VerticalScene2Breakthrough } from "./scenes/VerticalScene2Breakthrough";
import { VerticalScene3DualReality } from "./scenes/VerticalScene3DualReality";
import { VerticalScene4Browser } from "./scenes/VerticalScene4Browser";
import { VerticalScene5Integration } from "./scenes/VerticalScene5Integration";
import { VerticalScene6Outro } from "./scenes/VerticalScene6Outro";

export const VERTICAL_TOTAL_FRAMES = 2960; // 49.33s @ 60 FPS

export const VerticalFastuseVideo: React.FC = () => {
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        backgroundColor: "#030611",
        position: "relative",
      }}
    >
      {/* MASTER SOUNDTRACK & VOICEOVER */}
      <Audio src={staticFile("audio/master_audio.mp3")} volume={1.0} />

      {/* SCENE 1: THE CURSOR HIJACK WALL (0.0s - 6.0s) */}
      <Sequence from={0} durationInFrames={360}>
        <VerticalScene1Hijack />
      </Sequence>

      {/* SCENE 2: THE BREAKTHROUGH & BENCHMARKS (6.0s - 13.0s) */}
      <Sequence from={360} durationInFrames={420}>
        <VerticalScene2Breakthrough />
      </Sequence>

      {/* SCENE 3: DUAL REALITY & PARALLEL WORKSPACE (13.0s - 22.0s) */}
      <Sequence from={780} durationInFrames={540}>
        <VerticalScene3DualReality />
      </Sequence>

      {/* SCENE 4: REAL BRAVE/CHROME & SHADOW DOM (22.0s - 32.0s) */}
      <Sequence from={1320} durationInFrames={600}>
        <VerticalScene4Browser />
      </Sequence>

      {/* SCENE 5: ANTIGRAVITY & CLAUDE CODE INTEGRATION (32.0s - 40.0s) */}
      <Sequence from={1920} durationInFrames={480}>
        <VerticalScene5Integration />
      </Sequence>

      {/* SCENE 6: FINALE & GITHUB STAR CTA (40.0s - 49.33s) */}
      <Sequence from={2400} durationInFrames={560}>
        <VerticalScene6Outro />
      </Sequence>
    </div>
  );
};
