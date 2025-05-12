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
  const [errorDetails, setErrorDetails] = useState<string>("");
  const [logs, setLogs] = useState<string[]>([]);
  const videoRef = useRef<HTMLVideoElement>(null);

  const addLog = (message: string) => {
    setLogs((prev) => [...prev.slice(-4), message]); // Garde les 5 derniers logs
  };

  // Préparer l'URL de la vidéo en fonction de la plateforme
  const processedAnimationUrl = isWindows
    ? animationUrl
    : animationUrl.replace(/%2F/g, "/");

  const handleVideoEnded = useCallback(() => {
    if (currentPlayCount < playCount - 1) {
      setCurrentPlayCount((prev) => prev + 1);
      addLog(`Lecture ${currentPlayCount + 2}/${playCount}`);
      if (videoRef.current) {
        videoRef.current.currentTime = 0;
        videoRef.current.play();
      }
    } else {
      addLog("Lecture terminée");
      onClose();
    }
  }, [currentPlayCount, playCount, onClose]);

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      video.addEventListener("ended", handleVideoEnded);

      video.addEventListener("error", () => {
        const errorTypes = {
          1: "MEDIA_ERR_ABORTED - La lecture a été annulée",
          2: "MEDIA_ERR_NETWORK - Erreur réseau",
          3: "MEDIA_ERR_DECODE - Erreur de décodage",
          4: "MEDIA_ERR_SRC_NOT_SUPPORTED - Format non supporté ou source inaccessible",
        };

        const errorCode = video.error?.code || 0;
        const errorMessage =
          errorTypes[errorCode as keyof typeof errorTypes] || "Erreur inconnue";
        const detailedError = `${errorMessage} (Code ${errorCode}): ${video.error?.message}`;

        console.error("Erreur vidéo détaillée:", {
          code: errorCode,
          message: video.error?.message,
          currentSrc: video.currentSrc,
          readyState: video.readyState,
          networkState: video.networkState,
          error: video.error,
        });

        addLog(detailedError);
        setHasError(true);
        setErrorDetails(detailedError);
      });

      video.addEventListener("loadstart", () => {
        addLog(`Chargement vidéo démarré: ${processedAnimationUrl}`);
      });

      video.addEventListener("canplay", () => {
        addLog(`Vidéo prête à être lue: ${processedAnimationUrl}`);
      });

      // Add more event listeners for debugging
      video.addEventListener("loadedmetadata", () => {
        addLog("Métadonnées chargées");
      });

      video.addEventListener("stalled", () => {
        addLog("Lecture bloquée");
      });
    }

    return () => {
      if (video) {
        video.removeEventListener("ended", handleVideoEnded);
        video.removeEventListener("error", () => {});
        video.removeEventListener("loadstart", () => {});
        video.removeEventListener("canplay", () => {});
        video.removeEventListener("loadedmetadata", () => {});
        video.removeEventListener("stalled", () => {});
      }
    };
  }, [handleVideoEnded, processedAnimationUrl]);

  const togglePlayPause = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    if (videoRef.current) {
      if (isPlaying) {
        videoRef.current.pause();
        addLog("Lecture en pause");
      } else {
        videoRef.current.play();
        addLog("Lecture reprise");
      }
      setIsPlaying(!isPlaying);
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
        </div>
      ) : null}

      <video
        ref={videoRef}
        src={processedAnimationUrl}
        className={styles.videoPlayer}
        autoPlay
        onError={(e) => {
          const video = e.currentTarget;
          const errorMsg = `Erreur vidéo: Code ${video.error?.code}, ${video.error?.message}`;
          addLog(errorMsg);
          setErrorDetails(errorMsg);
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
