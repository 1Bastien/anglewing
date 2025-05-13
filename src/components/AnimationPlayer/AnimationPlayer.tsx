import React, { useState, useCallback, useEffect, useRef } from "react";
import styles from "./AnimationPlayer.module.css";
import PlayPauseButton from "../PlayPauseButton/PlayPauseButton";
import ReactPlayer from "react-player";

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
  const [isReady, setIsReady] = useState(false);
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const playerRef = useRef<ReactPlayer>(null);
  const isDuringPlayback = useRef(false);
  const leftButtonRef = useRef<HTMLButtonElement>(null);
  const autoRestartTimerRef = useRef<number | null>(null);
  const videoEndedTimeRef = useRef<number | null>(null);
  const maxRetries = 3;
  const [retryCount, setRetryCount] = useState(0);

  useEffect(() => {
    const fetchVideo = async () => {
      try {
        setIsLoading(true);
        
        const response = await fetch(animationUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! Status: ${response.status}`);
        }
        
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        
        setBlobUrl(url);
        setIsLoading(false);
      } catch (error) {
        setIsLoading(false);
      }
    };

    fetchVideo();

    return () => {
      if (blobUrl) {
        URL.revokeObjectURL(blobUrl);
      }
      if (autoRestartTimerRef.current) {
        clearTimeout(autoRestartTimerRef.current);
      }
    };
  }, [animationUrl]);

  const forceButtonClick = useCallback(() => {
    if (leftButtonRef.current) {
      leftButtonRef.current.click();
      return true;
    }
    return false;
  }, []);

  const simulatePlayButtonClick = useCallback(() => {
    setIsPlaying(false);
    setTimeout(() => {
      setIsPlaying(true);
    }, 50);
  }, []);

  const restartPlayback = useCallback(() => {
    if (playerRef.current) {
      playerRef.current.seekTo(0);
      
      simulatePlayButtonClick();
      
      forceButtonClick();
      
      setRetryCount(prev => prev + 1);
      return true;
    }
    return false;
  }, [simulatePlayButtonClick, forceButtonClick]);

  useEffect(() => {
    if (isReady && isPlaying && !isLoading && blobUrl) {
      const checkInterval = setInterval(() => {
        if (isPlaying && !isDuringPlayback.current && retryCount < maxRetries) {
          restartPlayback();
        }
      }, 1000);
      
      return () => clearInterval(checkInterval);
    }
  }, [isReady, isPlaying, isLoading, blobUrl, restartPlayback, retryCount]);

  const handleVideoEnded = useCallback(() => {
    videoEndedTimeRef.current = Date.now();
    isDuringPlayback.current = false;
    
    if (autoRestartTimerRef.current) {
      clearTimeout(autoRestartTimerRef.current);
    }
    
    const nextPlayCount = currentPlayCount + 1;
    
    if (nextPlayCount < playCount) {
      setCurrentPlayCount(nextPlayCount);
      setRetryCount(0);
      
      autoRestartTimerRef.current = window.setTimeout(() => {
        if (!restartPlayback()) {
          if (!forceButtonClick()) {
            simulatePlayButtonClick();
          }
        }
      }, 300);
      
      setTimeout(() => {
        if (!isDuringPlayback.current && nextPlayCount === currentPlayCount) {
          restartPlayback();
        }
      }, 1000);
    } else {
      onClose();
    }
  }, [currentPlayCount, playCount, onClose, restartPlayback, forceButtonClick, simulatePlayButtonClick]);

  const handleError = () => {

  };

  const togglePlayPause = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    if (isReady) {
      const newPlayingState = !isPlaying;
      setIsPlaying(newPlayingState);
      isDuringPlayback.current = newPlayingState;
    }
  };

  const handlePlay = () => {
    isDuringPlayback.current = true;
  };

  const handlePause = () => {
    isDuringPlayback.current = false;
  };

  const handleReady = () => {
    setIsReady(true);
    
    setIsPlaying(true);
    isDuringPlayback.current = true;
  };

  const handleProgress = (state: { played: number }) => {
    if (state.played > 0) {
      isDuringPlayback.current = true;
    }
  };

  return (
    <div
      className={styles.playerContainer}
      onClick={() => togglePlayPause({} as React.MouseEvent<HTMLButtonElement>)}
      style={{ backgroundColor: 'rgb(0, 0, 0)' }}
    >

      <div className={styles.videoWrapper}>
        {blobUrl && !isLoading && (
          <ReactPlayer
            ref={playerRef}
            url={blobUrl}
            playing={isPlaying}
            controls={false}
            width="100%"
            height="100%"
            style={{ backgroundColor: 'black' }}
            onEnded={handleVideoEnded}
            onError={handleError}
            onProgress={handleProgress}
            onBuffer={() => {
            }}
            onBufferEnd={() => {
            }}
            onReady={handleReady}
            onStart={() => {
              isDuringPlayback.current = true;
            }}
            onPause={handlePause}
            onPlay={handlePlay}
          />
        )}
      </div>

      <div className={styles.controlsContainer}>
        <PlayPauseButton
          ref={leftButtonRef}
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
