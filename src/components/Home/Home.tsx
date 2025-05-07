import React, { useEffect, useState, useCallback, useRef } from "react";
import styles from "./Home.module.css";
import Button from "../Button/Button";
import CloseButton from "../CloseButton/CloseButton";
import AnimationPlayer from "../AnimationPlayer/AnimationPlayer";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

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

  useEffect(() => {
    const getPublicPath = async () => {
      try {
        const path = await invoke<string>("get_public_folder_path");
        console.log("Chemin public récupéré:", path);

        // Normaliser le chemin pour Windows (remplacer les backslashes par des slashes)
        // Et s'assurer qu'il n'y a pas de slash à la fin
        const normalizedPath = path.replace(/\\/g, "/").replace(/\/$/, "");
        console.log("Chemin public normalisé:", normalizedPath);

        setPublicPath(normalizedPath);
      } catch (error) {
        console.error(
          "Erreur lors de la récupération du chemin public:",
          error
        );
      }
    };

    getPublicPath();
  }, []);

  const loadConfig = useCallback(async () => {
    if (!publicPath) {
      console.log(
        "Pas de chemin public disponible pour charger la configuration"
      );
      return;
    }

    try {
      // Construire le chemin avec les bons séparateurs
      const filePath = `${publicPath}/config.json`;
      console.log("Chemin du fichier de configuration:", filePath);

      // Utilisation de encodeURI pour gérer les espaces et caractères spéciaux
      const configPath = convertFileSrc(
        encodeURI(filePath).replace(/#/g, "%23")
      );
      console.log("URL du fichier de configuration:", configPath);

      const response = await fetch(configPath);

      if (!response.ok) {
        throw new Error(
          `Erreur HTTP ${response.status}: ${response.statusText}`
        );
      }

      const data = await response.json();
      console.log("Configuration chargée:", data);

      if (JSON.stringify(data) === JSON.stringify(config)) {
        return;
      }

      setConfig(data);

      if (data.background) {
        // Construire le chemin avec les bons séparateurs
        const bgFilePath = `${publicPath}/backgrounds/${data.background.file}`;
        console.log("Chemin de l'image de fond:", bgFilePath);

        // Utilisation de encodeURI pour gérer les espaces et caractères spéciaux
        const backgroundUrl = convertFileSrc(
          encodeURI(bgFilePath).replace(/#/g, "%23")
        );
        console.log("URL de l'image de fond:", backgroundUrl);

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
    if (publicPath) {
      loadConfig();
      const intervalId = setInterval(loadConfig, CONFIG_CHECK_INTERVAL);
      return () => clearInterval(intervalId);
    }
  }, [loadConfig, publicPath]);

  const handleAnimationSelect = (animationId: number) => {
    if (config && publicPath) {
      const animation = config.animations.find((a) => a.id === animationId);
      if (animation) {
        // Construire le chemin avec les bons séparateurs
        const animPath = `${publicPath}/animations/${animation.file}`;
        console.log("Chemin de l'animation:", animPath);

        // Utilisation de encodeURI pour gérer les espaces et caractères spéciaux
        const animationUrl = convertFileSrc(
          encodeURI(animPath).replace(/#/g, "%23")
        );
        console.log("URL de l'animation:", animationUrl);
        console.log("Fichier d'animation:", animation.file);

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
