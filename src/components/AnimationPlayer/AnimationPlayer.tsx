import React, { useState, useCallback } from "react";
import styles from "./AnimationPlayer.module.css";
import PlayPauseButton from "../PlayPauseButton/PlayPauseButton";
import ReactPlayer from "react-player";

interface AnimationPlayerProps {
  animationUrl: string;
  playCount: number;
  onClose: () => void;
  isWindows?: boolean;
}

// Détection de la plateforme une seule fois en dehors du composant
const userAgent = navigator.userAgent.toLowerCase();
const isLinuxPlatform = userAgent.includes("linux");

const AnimationPlayer: React.FC<AnimationPlayerProps> = ({
  animationUrl,
  playCount,
  onClose,
  isWindows = false,
}) => {
  const [isPlaying, setIsPlaying] = useState(true);
  const [currentPlayCount, setCurrentPlayCount] = useState(0);
  const [hasError, setHasError] = useState(false);
  const [errorDetails, setErrorDetails] = useState<string>("");
  const [logs, setLogs] = useState<string[]>([]);

  const addLog = (message: string) => {
    setLogs((prev) => [...prev.slice(-4), message]);
  };

  // Préparer l'URL de la vidéo en fonction de la plateforme
  const processedAnimationUrl = isWindows
    ? animationUrl
    : isLinuxPlatform
    ? `file://${animationUrl}`
    : animationUrl.replace(/%2F/g, "/");

  const handleVideoEnded = useCallback(() => {
    if (currentPlayCount < playCount - 1) {
      setCurrentPlayCount((prev) => prev + 1);
      addLog(`Lecture ${currentPlayCount + 2}/${playCount}`);
      setIsPlaying(true);
    } else {
      addLog("Lecture terminée");
      onClose();
    }
  }, [currentPlayCount, playCount, onClose]);

  const handleError = (error: any) => {
    const errorMessage = `Erreur de lecture: ${
      error?.message || "Erreur inconnue"
    }`;
    addLog(errorMessage);
    setHasError(true);
    setErrorDetails(errorMessage);
  };

  const togglePlayPause = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    setIsPlaying(!isPlaying);
    addLog(isPlaying ? "Lecture en pause" : "Lecture reprise");
  };

  return (
    <div
      className={styles.playerContainer}
      onClick={() => togglePlayPause({} as React.MouseEvent<HTMLButtonElement>)}
    >
      <div className={styles.debugInfo}>
        <h3>Informations lecture</h3>
        <p>État: {isPlaying ? "En lecture" : "En pause"}</p>
        <p>
          Lecture: {currentPlayCount + 1}/{playCount}
        </p>
        <p>
          Plateforme:{" "}
          {isLinuxPlatform ? "Linux" : isWindows ? "Windows" : "macOS"}
        </p>
        <p>URL vidéo: {processedAnimationUrl}</p>

        <h3>Logs lecture</h3>
        <div className={styles.logs}>
          {logs.map((log, index) => (
            <p key={index} className={styles.logEntry}>
              {log}
            </p>
          ))}
        </div>
      </div>

      {hasError ? (
        <div className={styles.errorMessage}>
          <p>Erreur lors du chargement de la vidéo</p>
          <p>Détails: {errorDetails}</p>
          <p>URL utilisée: {processedAnimationUrl}</p>
          <p>
            Plateforme:{" "}
            {isLinuxPlatform ? "Linux" : isWindows ? "Windows" : "macOS"}
          </p>
        </div>
      ) : null}

      <div className={styles.videoWrapper}>
        <ReactPlayer
          url={processedAnimationUrl}
          playing={isPlaying}
          controls={false}
          width="100%"
          height="100%"
          onEnded={handleVideoEnded}
          onError={handleError}
          onBuffer={() => addLog("Mise en mémoire tampon...")}
          onBufferEnd={() => addLog("Lecture prête")}
          onReady={() => addLog("Lecteur prêt")}
          onStart={() => addLog("Lecture démarrée")}
          config={{
            file: {
              attributes: {
                playsInline: true,
              },
            },
          }}
        />
      </div>

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
