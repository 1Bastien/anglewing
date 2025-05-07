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

  // État pour le débogage
  const [debugInfo, setDebugInfo] = useState<string[]>([]);
  const [showDebug, setShowDebug] = useState(true);

  // Fonction d'utilitaire pour ajouter des messages de débogage
  const addDebugMessage = useCallback((message: string) => {
    console.log(message);
    setDebugInfo((prev) => {
      const newMessages = [...prev, message];
      // Garder seulement les 15 derniers messages pour éviter de surcharger l'écran
      if (newMessages.length > 15) {
        return newMessages.slice(newMessages.length - 15);
      }
      return newMessages;
    });
  }, []);

  useEffect(() => {
    const getPublicPath = async () => {
      try {
        addDebugMessage("Récupération du chemin public...");
        const path = await invoke<string>("get_public_folder_path");
        addDebugMessage(`Chemin public récupéré: ${path}`);

        // Normaliser le chemin pour Windows (remplacer les backslashes par des slashes)
        // Et s'assurer qu'il n'y a pas de slash à la fin
        const normalizedPath = path.replace(/\\/g, "/").replace(/\/$/, "");
        addDebugMessage(`Chemin public normalisé: ${normalizedPath}`);

        setPublicPath(normalizedPath);
      } catch (error) {
        addDebugMessage(`ERREUR: Récupération du chemin public: ${error}`);
      }
    };

    getPublicPath();
  }, [addDebugMessage]);

  const loadConfig = useCallback(async () => {
    if (!publicPath) {
      addDebugMessage(
        "Pas de chemin public disponible pour charger la configuration"
      );
      return;
    }

    try {
      // Construire le chemin avec les bons séparateurs
      const filePath = `${publicPath}/config.json`;
      addDebugMessage(`Chemin config.json: ${filePath}`);

      // Utilisation de encodeURI pour gérer les espaces et caractères spéciaux
      const encodedPath = encodeURI(filePath).replace(/#/g, "%23");
      const configPath = convertFileSrc(encodedPath);
      addDebugMessage(`URL convertie: ${configPath}`);

      addDebugMessage("Tentative de chargement de config.json...");
      const response = await fetch(configPath);
      addDebugMessage("Fetch config.json réussi!");

      if (!response.ok) {
        throw new Error(
          `Erreur HTTP ${response.status}: ${response.statusText}`
        );
      }

      const data = await response.json();
      addDebugMessage("Configuration JSON parsée avec succès");

      if (JSON.stringify(data) === JSON.stringify(config)) {
        return;
      }

      setConfig(data);

      if (data.background) {
        // Construire le chemin avec les bons séparateurs
        const bgFilePath = `${publicPath}/backgrounds/${data.background.file}`;
        addDebugMessage(`Chemin image de fond: ${bgFilePath}`);

        // Utilisation de encodeURI pour gérer les espaces et caractères spéciaux
        const encodedBgPath = encodeURI(bgFilePath).replace(/#/g, "%23");
        const backgroundUrl = convertFileSrc(encodedBgPath);
        addDebugMessage(`URL image de fond: ${backgroundUrl}`);

        setBackgroundStyle({
          backgroundImage: `url(${backgroundUrl})`,
          backgroundSize: "cover",
          backgroundPosition: "center",
          backgroundRepeat: "no-repeat",
        });
      }
    } catch (error) {
      addDebugMessage(`ERREUR: Chargement de la configuration: ${error}`);
    }
  }, [publicPath, config, addDebugMessage]);

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
        addDebugMessage(`Chemin animation: ${animPath}`);

        // Utilisation de encodeURI pour gérer les espaces et caractères spéciaux
        const encodedAnimPath = encodeURI(animPath).replace(/#/g, "%23");
        const animationUrl = convertFileSrc(encodedAnimPath);
        addDebugMessage(`URL animation: ${animationUrl}`);
        addDebugMessage(`Fichier d'animation: ${animation.file}`);

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

      {/* Affichage des informations de débogage */}
      {showDebug && (
        <div className={styles.debugPanel}>
          <div className={styles.debugHeader}>
            <h3>Informations de débogage</h3>
            <button
              onClick={() => setShowDebug(false)}
              className={styles.debugCloseButton}
            >
              Fermer
            </button>
          </div>
          <div className={styles.debugContent}>
            {debugInfo.map((message, index) => (
              <div key={index} className={styles.debugMessage}>
                {message.includes("ERREUR") ? (
                  <div className={styles.debugError}>{message}</div>
                ) : (
                  <div>{message}</div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default Home;
