import React, { useRef, useState, useEffect, useCallback } from "react";
import styles from "./AnimationPlayer.module.css";
import PlayPauseButton from "../PlayPauseButton/PlayPauseButton";

interface AnimationPlayerProps {
  animationUrl: string;
  playCount: number;
  onClose: () => void;
}

const AnimationPlayer: React.FC<AnimationPlayerProps> = ({
  animationUrl,
  playCount,
  onClose,
}) => {
  const [isPlaying, setIsPlaying] = useState(true);
  const [currentPlayCount, setCurrentPlayCount] = useState(0);
  const videoRef = useRef<HTMLVideoElement>(null);

  const handleVideoEnded = useCallback(() => {
    if (currentPlayCount < playCount - 1) {
      setCurrentPlayCount((prev) => prev + 1);
      if (videoRef.current) {
        videoRef.current.currentTime = 0;
        videoRef.current.play();
      }
    } else {
      onClose();
    }
  }, [currentPlayCount, playCount, onClose]);

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      video.addEventListener("ended", handleVideoEnded);
    }

    return () => {
      if (video) {
        video.removeEventListener("ended", handleVideoEnded);
      }
    };
  }, [handleVideoEnded]);

  const togglePlayPause = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    if (videoRef.current) {
      if (isPlaying) {
        videoRef.current.pause();
      } else {
        videoRef.current.play();
      }
      setIsPlaying(!isPlaying);
    }
  };

  return (
    <div
      className={styles.playerContainer}
      onClick={() => togglePlayPause({} as React.MouseEvent<HTMLButtonElement>)}
    >
      <video
        ref={videoRef}
        src={animationUrl}
        className={styles.videoPlayer}
        autoPlay
      />
      <div className={styles.controlsContainer}>
        <PlayPauseButton
          isPlaying={isPlaying}
          onClick={togglePlayPause}
          className={styles.leftButton}
        />

        <PlayPauseButton
          isPlaying={isPlaying}
          onClick={togglePlayPause}
          className={styles.rightButton}
        />
      </div>
    </div>
  );
};

export default AnimationPlayer;
