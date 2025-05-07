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
  const inactivityTimerRef = useRef<number | null>(null);
  const [isWindows, setIsWindows] = useState(false);

  // Détecter la plateforme au chargement - version simplifiée sans dépendance à os
  useEffect(() => {
    const detectPlatform = () => {
      try {
        // Détection simplifiée - vérifier si navigator.userAgent contient "Windows"
        const userAgent = navigator.userAgent.toLowerCase();
        const isWin =
          userAgent.includes("windows") ||
          userAgent.includes("win32") ||
          userAgent.includes("win64");
        setIsWindows(isWin);
      } catch (error) {
        console.error("Erreur lors de la détection de la plateforme:", error);
      }
    };

    detectPlatform();
  }, []);

  // Récupérer le chemin public
  useEffect(() => {
    const getPublicPath = async () => {
      try {
        // Pour toutes les plateformes, on utilise get_public_folder_path
        const path = await invoke<string>("get_public_folder_path");

        // Normaliser le chemin pour Windows (remplacer les backslashes par des slashes)
        // Et s'assurer qu'il n'y a pas de slash à la fin
        const normalizedPath = path.replace(/\\/g, "/").replace(/\/$/, "");
        setPublicPath(normalizedPath);
      } catch (error) {
        console.error(
          "Erreur lors de la récupération du chemin public:",
          error
        );
      }
    };

    getPublicPath();
  }, [isWindows]);

  const loadConfig = useCallback(async () => {
    if (!publicPath) {
      return;
    }

    try {
      let configUrl;
      let response;

      // Méthode pour toutes les plateformes: utiliser convertFileSrc
      const filePath = `${publicPath}/config.json`;
      const encodedPath = encodeURI(filePath).replace(/#/g, "%23");
      configUrl = convertFileSrc(encodedPath);
      response = await fetch(configUrl);

      if (!response.ok) {
        throw new Error(
          `Erreur HTTP ${response.status}: ${response.statusText}`
        );
      }

      const data = await response.json();

      if (JSON.stringify(data) === JSON.stringify(config)) {
        return;
      }

      setConfig(data);

      if (data.background) {
        // Méthode pour toutes les plateformes: utiliser convertFileSrc
        const bgFilePath = `${publicPath}/backgrounds/${data.background.file}`;
        const encodedBgPath = encodeURI(bgFilePath).replace(/#/g, "%23");
        const backgroundUrl = convertFileSrc(encodedBgPath);

        setBackgroundStyle({
          backgroundImage: `url(${backgroundUrl})`,
          backgroundSize: "cover",
          backgroundPosition: "center",
          backgroundRepeat: "no-repeat",
        });
      }
    } catch (error) {
      console.error("Erreur lors du chargement de la configuration:", error);
    }
  }, [publicPath, config]);

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
        // Méthode pour toutes les plateformes: utiliser convertFileSrc
        const animPath = `${publicPath}/animations/${animation.file}`;
        const encodedAnimPath = encodeURI(animPath).replace(/#/g, "%23");
        const animationUrl = convertFileSrc(encodedAnimPath);

        setSelectedAnimation({
          ...animation,
          file: animationUrl,
        });
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
      <div className={styles.backgroundImage} style={backgroundStyle}></div>
      <div className={styles.buttonsContainer}>
        {config?.animations.map((animation) => (
          <Button
            key={animation.id}
            text={animation.title}
            onClick={() => handleAnimationSelect(animation.id)}
          />
        ))}
      </div>

      {selectedAnimation && (
        <AnimationPlayer
          animationUrl={selectedAnimation.file}
          playCount={selectedAnimation.playCount}
          onClose={closeAnimationPlayer}
        />
      )}
    </div>
  );
};

export default Home;
