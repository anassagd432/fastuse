import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";
import { THEME } from "../theme";

interface WordItem {
  text: string;
  isAccent?: boolean;
  color?: string;
}

interface KineticWordsProps {
  words: WordItem[];
  startFrame?: number;
  stagger?: number;
  fontSize?: number;
  lineHeight?: number;
}

export const KineticWords: React.FC<KineticWordsProps> = ({
  words,
  startFrame = 0,
  stagger = 4,
  fontSize = 90,
  lineHeight = 1.25,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "center",
        alignItems: "center",
        gap: `${fontSize * 0.28}px`,
        maxWidth: 1900,
        lineHeight,
      }}
    >
      {words.map((item, i) => {
        const wordDelay = startFrame + i * stagger;
        const wordSpring = spring({
          frame: frame - wordDelay,
          fps,
          config: { damping: 12, stiffness: 220 },
        });

        const activeColor = item.color
          ? item.color
          : item.isAccent
          ? THEME.cyan
          : THEME.text;

        return (
          <span
            key={i}
            style={{
              fontFamily: THEME.fontFamily,
              fontSize,
              fontWeight: 900,
              color: activeColor,
              letterSpacing: -1.5,
              display: "inline-block",
              transform: `scale(${wordSpring}) translateY(${
                (1 - wordSpring) * 35
              }px)`,
              opacity: wordSpring,
              textShadow: item.isAccent
                ? `0 0 35px ${THEME.blueGlow}`
                : "0 10px 30px rgba(0, 0, 0, 0.7)",
            }}
          >
            {item.text}
          </span>
        );
      })}
    </div>
  );
};
