import React, { useEffect, useState, useCallback, useRef } from "react";
import styles from "./Home.module.css";
import Button from "../Button/Button";
import CloseButton from "../CloseButton/CloseButton";
import AnimationPlayer from "../AnimationPlayer/AnimationPlayer";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

interface AnimationConfig {
  id: number;
  title: string;
  file: string;
  playCount: number;
}

interface BackgroundConfig {
  file: string;
}

interface Config {
  background: BackgroundConfig;
  animations: AnimationConfig[];
  inactivity_timeout_in_minutes: number;
}

const CONFIG_CHECK_INTERVAL = 30000;

const Home: React.FC = () => {
  const [backgroundStyle, setBackgroundStyle] = useState({});
  const [selectedAnimation, setSelectedAnimation] =
    useState<AnimationConfig | null>(null);
  const [config, setConfig] = useState<Config | null>(null);
  const [publicPath, setPublicPath] = useState<string | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const inactivityTimerRef = useRef<number | null>(null);
  const [isWindows, setIsWindows] = useState(false);

  const addLog = (message: string) => {
    setLogs((prev) => [...prev.slice(-9), message]); // Garde les 10 derniers logs
  };

  // Détecter la plateforme au chargement
  useEffect(() => {
    const detectPlatform = () => {
      try {
        const userAgent = navigator.userAgent.toLowerCase();
        const isWin =
          userAgent.includes("windows") ||
          userAgent.includes("win32") ||
          userAgent.includes("win64");
        setIsWindows(isWin);
        addLog(`Plateforme détectée: ${isWin ? "Windows" : "Non-Windows"}`);
      } catch (error) {
        addLog(`Erreur détection plateforme: ${error}`);
      }
    };

    detectPlatform();
  }, []);

  // Récupérer le chemin public
  useEffect(() => {
    const getPublicPath = async () => {
      try {
        const path = await invoke<string>("get_public_folder_path");
        const normalizedPath = path.replace(/\\/g, "/").replace(/\/$/, "");
        setPublicPath(normalizedPath);
        addLog(`Chemin public obtenu: ${normalizedPath}`);
      } catch (error) {
        addLog(`Erreur chemin public: ${error}`);
      }
    };

    getPublicPath();
  }, [isWindows]);

  const loadConfig = useCallback(async () => {
    if (!publicPath) {
      return;
    }

    try {
      const filePath = `${publicPath}/config.json`;
      const encodedPath = isWindows
        ? encodeURI(filePath).replace(/#/g, "%23")
        : filePath;
      const configUrl = convertFileSrc(encodedPath);

      addLog(`Chargement config depuis: ${configUrl}`);
      const response = await fetch(configUrl);

      if (!response.ok) {
        const errorMsg = `Erreur HTTP ${response.status}: ${response.statusText}`;
        setConfigError(errorMsg);
        addLog(`Erreur config: ${errorMsg}`);
        throw new Error(errorMsg);
      }

      const data = await response.json();
      addLog("Configuration chargée avec succès");

      if (JSON.stringify(data) === JSON.stringify(config)) {
        return;
      }

      setConfig(data);
      setConfigError(null);

      if (data.background) {
        const bgFilePath = `${publicPath}/backgrounds/${data.background.file}`;
        const encodedBgPath = isWindows
          ? encodeURI(bgFilePath).replace(/#/g, "%23")
          : bgFilePath;
        const backgroundUrl = convertFileSrc(encodedBgPath);

        addLog(`Arrière-plan chargé: ${data.background.file}`);
        setBackgroundStyle({
          backgroundImage: `url(${backgroundUrl})`,
          backgroundSize: "cover",
          backgroundPosition: "center",
          backgroundRepeat: "no-repeat",
        });
      }
    } catch (error) {
      const errorMsg = `Erreur config: ${error}`;
      addLog(errorMsg);
      setConfigError(errorMsg);
    }
  }, [publicPath, config, isWindows]);

  const resetInactivityTimer = useCallback(() => {
    if (inactivityTimerRef.current !== null) {
      window.clearTimeout(inactivityTimerRef.current);
    }

    invoke("reset_inactivity_timer").catch((err: Error) => {
      console.error("Failed to reset inactivity timer:", err);
    });

    inactivityTimerRef.current = window.setTimeout(async () => {
      if (!selectedAnimation) {
        try {
          await invoke("put_system_to_sleep");
        } catch (error) {
          console.error("Failed to put system to sleep:", error);
        }
      }
    }, (config?.inactivity_timeout_in_minutes || 10) * 60 * 1000);
  }, [selectedAnimation, config]);

  useEffect(() => {
    resetInactivityTimer();

    const activityEvents = [
      "mousedown",
      "mousemove",
      "keypress",
      "touchstart",
      "touchmove",
      "click",
    ];

    const handleUserActivity = () => {
      resetInactivityTimer();
    };

    activityEvents.forEach((eventName) => {
      document.addEventListener(eventName, handleUserActivity);
    });

    return () => {
      if (inactivityTimerRef.current !== null) {
        window.clearTimeout(inactivityTimerRef.current);
      }

      activityEvents.forEach((eventName) => {
        document.removeEventListener(eventName, handleUserActivity);
      });
    };
  }, [resetInactivityTimer]);

  useEffect(() => {
    loadConfig();
    const intervalId = setInterval(loadConfig, CONFIG_CHECK_INTERVAL);
    return () => clearInterval(intervalId);
  }, [loadConfig]);

  const handleAnimationSelect = (animationId: number) => {
    if (config) {
      const animation = config.animations.find((a) => a.id === animationId);
      if (animation) {
        const animPath = `${publicPath}/animations/${animation.file}`;
        const encodedAnimPath = isWindows
          ? encodeURI(animPath).replace(/#/g, "%23")
          : animPath;
        const animationUrl = convertFileSrc(encodedAnimPath);

        setSelectedAnimation({
          ...animation,
          file: animationUrl,
        });
        addLog(`Animation chargée: ${animation.file}`);
      }
    }
  };

  const closeAnimationPlayer = () => {
    setSelectedAnimation(null);
    resetInactivityTimer();
  };

  return (
    <div className={styles.container}>
      <CloseButton />

      {/* Debug information */}
      <div className={styles.debugInfo}>
        <h3>Informations système</h3>
        <p>Chemin public: {publicPath}</p>
        <p>Plateforme Windows: {isWindows ? "Oui" : "Non"}</p>
        {configError && <p className={styles.error}>Erreur: {configError}</p>}

        <h3>Logs système</h3>
        <div className={styles.logs}>
          {logs.map((log, index) => (
            <p key={index} className={styles.logEntry}>
              {log}
            </p>
          ))}
        </div>
      </div>

      <div className={styles.backgroundImage} style={backgroundStyle}></div>
      <div className={styles.buttonsContainer}>
        {config?.animations.map((animation) => (
          <Button
            key={animation.id}
            text={animation.title}
            onClick={() => {
              addLog(`Animation sélectionnée: ${animation.title}`);
              handleAnimationSelect(animation.id);
            }}
          />
        ))}
      </div>

      {selectedAnimation && (
        <AnimationPlayer
          animationUrl={selectedAnimation.file}
          playCount={selectedAnimation.playCount}
          onClose={() => {
            addLog("Lecture animation terminée");
            closeAnimationPlayer();
          }}
          isWindows={isWindows}
        />
      )}
    </div>
  );
};

export default Home;
