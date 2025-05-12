import React, { useRef, useState, useEffect, useCallback } from "react";
import styles from "./AnimationPlayer.module.css";
import PlayPauseButton from "../PlayPauseButton/PlayPauseButton";

interface AnimationPlayerProps {
  animationUrl: string;
  playCount: number;
  onClose: () => void;
  isWindows?: boolean;
}

const AnimationPlayer: React.FC<AnimationPlayerProps> = ({
  animationUrl,
  playCount,
  onClose,
  isWindows = false,
}) => {
  const [isPlaying, setIsPlaying] = useState(true);
  const [currentPlayCount, setCurrentPlayCount] = useState(0);
  const [hasError, setHasError] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  // Préparer l'URL de la vidéo en fonction de la plateforme
  const processedAnimationUrl = isWindows
    ? animationUrl
    : animationUrl.replace(/%2F/g, "/");

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

      // Ajouter des gestionnaires d'événements pour déboguer
      video.addEventListener("error", (e) => {
        console.error("Erreur de chargement vidéo:", e);
        console.error("Code d'erreur:", video.error?.code);
        console.error("Message d'erreur:", video.error?.message);
        setHasError(true);
      });

      video.addEventListener("loadstart", () => {
        console.log("Chargement de la vidéo commencé");
      });

      video.addEventListener("canplay", () => {
        console.log("La vidéo peut être lue");
      });
    }

    return () => {
      if (video) {
        video.removeEventListener("ended", handleVideoEnded);
        video.removeEventListener("error", () => {});
        video.removeEventListener("loadstart", () => {});
        video.removeEventListener("canplay", () => {});
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

  console.log(
    "Tentative de lecture de la vidéo depuis:",
    processedAnimationUrl
  );

  return (
    <div
      className={styles.playerContainer}
      onClick={() => togglePlayPause({} as React.MouseEvent<HTMLButtonElement>)}
    >
      {hasError ? (
        <div className={styles.errorMessage}>
          Erreur lors du chargement de la vidéo. Le chemin pourrait être
          incorrect.
        </div>
      ) : null}

      <video
        ref={videoRef}
        src={processedAnimationUrl}
        className={styles.videoPlayer}
        autoPlay
        onError={(e) => {
          console.error("Erreur de chargement de la vidéo (JSX):", e);
        }}
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
