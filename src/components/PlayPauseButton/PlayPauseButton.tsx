import React, { forwardRef } from "react";
import styles from "./PlayPauseButton.module.css";

interface PlayPauseButtonProps {
  isPlaying: boolean;
  onClick: (e: React.MouseEvent<HTMLButtonElement>) => void;
  className?: string;
}

const PlayPauseButton = forwardRef<HTMLButtonElement, PlayPauseButtonProps>(
  ({ isPlaying, onClick, className = "" }, ref) => {
    return (
      <button
        ref={ref}
        className={`${styles.playPauseButton} ${className}`}
        onClick={onClick}
        aria-label={isPlaying ? "Pause" : "Play"}
      >
        {isPlaying ? (
          <svg
            width="40"
            height="40"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <rect x="7" y="4" width="3" height="16" rx="1" fill="white" />
            <rect x="14" y="4" width="3" height="16" rx="1" fill="white" />
          </svg>
        ) : (
          <svg
            width="40"
            height="40"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path d="M7 4L19 12L7 20V4Z" fill="white" />
          </svg>
        )}
      </button>
    );
  }
);

PlayPauseButton.displayName = "PlayPauseButton";

export default PlayPauseButton;
