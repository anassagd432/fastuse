import React from "react";
import { Composition, Folder } from "remotion";
import {
  VerticalFastuseVideo,
  VERTICAL_TOTAL_FRAMES,
} from "./VerticalFastuseVideo";

export const RemotionRoot: React.FC = () => {
  return (
    <Folder name="Fastuse">
      {/* 9:16 Phone Ratio - Standard 1080p Vertical (1080x1920 @ 60 FPS) */}
      {/* Fits comfortably on 1080p & laptop screens while remaining crystal clear */}
      <Composition
        id="FastuseVertical"
        component={VerticalFastuseVideo}
        durationInFrames={VERTICAL_TOTAL_FRAMES}
        fps={60}
        width={1080}
        height={1920}
      />

      {/* 9:16 Phone Ratio - 4K Ultra HD Vertical Master (2160x3840 @ 60 FPS) */}
      <Composition
        id="FastuseVertical4K"
        component={VerticalFastuseVideo}
        durationInFrames={VERTICAL_TOTAL_FRAMES}
        fps={60}
        width={2160}
        height={3840}
      />
    </Folder>
  );
};
