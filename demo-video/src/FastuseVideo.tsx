import React from "react";
import { Audio, Series, staticFile } from "remotion";
import { Scene1Hijack } from "./scenes/Scene1Hijack";
import { Scene2Breakthrough } from "./scenes/Scene2Breakthrough";
import { Scene3DualReality } from "./scenes/Scene3DualReality";
import { Scene4Browser } from "./scenes/Scene4Browser";
import { Scene5Integration } from "./scenes/Scene5Integration";
import { Scene6Outro } from "./scenes/Scene6Outro";

export const TOTAL_DURATION_IN_FRAMES = 3240; // 54 seconds @ 60 FPS

export const FastuseVideo: React.FC = () => {
  return (
    <div style={{ flex: 1, backgroundColor: "#07090E" }}>
      {/* ElevenLabs Voiceover Audio Track */}
      <Audio src={staticFile("audio/voiceover.mp3")} volume={1.0} />

      {/* Main Narrative Scene Sequence */}
      <Series>
        {/* Scene 1: The Problem / Hijack (8s = 480 frames) */}
        <Series.Sequence durationInFrames={480}>
          <Scene1Hijack />
        </Series.Sequence>

        {/* Scene 2: The Breakthrough / fastuse + Latency (10s = 600 frames) */}
        <Series.Sequence durationInFrames={600}>
          <Scene2Breakthrough />
        </Series.Sequence>

        {/* Scene 3: Dual Reality / Background Execution (11s = 660 frames) */}
        <Series.Sequence durationInFrames={660}>
          <Scene3DualReality />
        </Series.Sequence>

        {/* Scene 4: Real Browser Control (10s = 600 frames) */}
        <Series.Sequence durationInFrames={600}>
          <Scene4Browser />
        </Series.Sequence>

        {/* Scene 5: Antigravity One-Prompt Install (7s = 420 frames) */}
        <Series.Sequence durationInFrames={420}>
          <Scene5Integration />
        </Series.Sequence>

        {/* Scene 6: Outro / Take Back Your Cursor (8s = 480 frames) */}
        <Series.Sequence durationInFrames={480}>
          <Scene6Outro />
        </Series.Sequence>
      </Series>
    </div>
  );
};
