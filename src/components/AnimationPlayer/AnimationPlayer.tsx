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
  const [isReady, setIsReady] = useState(false);
  const [playerState, setPlayerState] = useState<string>("Non initialisé");

  const addLog = (message: string) => {
    setLogs((prev) => [
      ...prev.slice(-4),
      `${new Date().toISOString().split("T")[1]} - ${message}`,
    ]);
  };

  // L'URL est déjà traitée par convertFileSrc dans Home.tsx
  const processedAnimationUrl = animationUrl;

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
    let errorMessage = "Erreur inconnue";

    // Détails de l'erreur
    if (error) {
      if (typeof error === "string") {
        errorMessage = error;
      } else if (error.message) {
        errorMessage = error.message;
      } else {
        try {
          errorMessage = JSON.stringify(error);
        } catch {
          errorMessage = "Erreur non sérialisable";
        }
      }
    }

    // Ajouter des informations sur l'état du lecteur
    const errorDetails = `
      Erreur de lecture: ${errorMessage}
      État du lecteur: ${isReady ? "Prêt" : "Non prêt"}
      État de lecture: ${isPlaying ? "En lecture" : "En pause"}
      État du player: ${playerState}
      Compteur: ${currentPlayCount + 1}/${playCount}
      URL: ${processedAnimationUrl}
    `.trim();

    addLog(`Erreur détectée: ${errorMessage}`);
    setHasError(true);
    setErrorDetails(errorDetails);
  };

  const togglePlayPause = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    if (isReady) {
      setIsPlaying(!isPlaying);
      addLog(isPlaying ? "Lecture en pause" : "Lecture reprise");
    } else {
      addLog("Lecteur pas encore prêt");
    }
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
        <p>Plateforme: {isWindows ? "Windows" : "Unix"}</p>
        <p>URL vidéo: {processedAnimationUrl}</p>
        <p>Lecteur prêt: {isReady ? "Oui" : "Non"}</p>
        <p>État du player: {playerState}</p>

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
          <p style={{ whiteSpace: "pre-line" }}>Détails: {errorDetails}</p>
          <p>URL utilisée: {processedAnimationUrl}</p>
          <p>Plateforme: {isWindows ? "Windows" : "Unix"}</p>
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
          onBuffer={() => {
            setPlayerState("Mise en mémoire tampon");
            addLog("Mise en mémoire tampon...");
          }}
          onBufferEnd={() => {
            setPlayerState("Tampon prêt");
            addLog("Lecture prête");
          }}
          onReady={() => {
            setIsReady(true);
            setPlayerState("Prêt");
            addLog("Lecteur prêt");
          }}
          onStart={() => {
            setPlayerState("En lecture");
            addLog("Lecture démarrée");
          }}
          onPause={() => {
            setPlayerState("En pause");
            addLog("Lecture en pause");
          }}
          onPlay={() => {
            setPlayerState("En lecture");
            addLog("Lecture en cours");
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
