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

  // État pour le débogage et la plateforme
  const [debugInfo, setDebugInfo] = useState<string[]>([]);
  const [showDebug, setShowDebug] = useState(true);
  const [isWindows, setIsWindows] = useState(false);

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
        addDebugMessage(`Plateforme détectée via userAgent, Windows: ${isWin}`);
      } catch (error) {
        console.error("Erreur lors de la détection de la plateforme:", error);
        addDebugMessage(
          "Impossible de détecter la plateforme, utilisation du mode standard"
        );
      }
    };

    detectPlatform();
  }, [addDebugMessage]);

  // Récupérer le chemin public
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

        if (isWindows) {
          addDebugMessage(
            "Windows détecté: utilisation du chemin direct ./_up_/public/"
          );
        } else {
          addDebugMessage(
            "Mac/Linux détecté: utilisation de la méthode standard"
          );
        }
      } catch (error) {
        addDebugMessage(`ERREUR: Récupération du chemin public: ${error}`);
      }
    };

    getPublicPath();
  }, [addDebugMessage, isWindows]);

  const loadConfig = useCallback(async () => {
    if (!isWindows && !publicPath) {
      addDebugMessage(
        "Pas de chemin public disponible pour charger la configuration"
      );
      return;
    }

    try {
      let configUrl;
      let response;

      if (isWindows) {
        // Méthode pour Windows: chemin direct
        configUrl = "./_up_/public/config.json";
        addDebugMessage(`Windows: utilisation du chemin direct: ${configUrl}`);
      } else {
        // Méthode pour Mac/Linux: utiliser convertFileSrc
        const filePath = `${publicPath}/config.json`;
        addDebugMessage(`Mac/Linux: chemin config.json: ${filePath}`);

        const encodedPath = encodeURI(filePath).replace(/#/g, "%23");
        configUrl = convertFileSrc(encodedPath);
        addDebugMessage(`Mac/Linux: URL convertie: ${configUrl}`);
      }

      addDebugMessage(
        `Tentative de chargement de config.json depuis: ${configUrl}`
      );
      response = await fetch(configUrl);

      if (!response.ok) {
        const errMsg = `Erreur HTTP ${response.status}: ${response.statusText}`;
        addDebugMessage(`ERREUR HTTP: ${errMsg}`);
        throw new Error(errMsg);
      }

      addDebugMessage("Fetch config.json réussi!");
      const data = await response.json();
      addDebugMessage("Configuration JSON parsée avec succès");

      if (JSON.stringify(data) === JSON.stringify(config)) {
        return;
      }

      setConfig(data);

      if (data.background) {
        let backgroundUrl;

        if (isWindows) {
          // Méthode pour Windows: chemin direct
          backgroundUrl = `./_up_/public/backgrounds/${data.background.file}`;
          addDebugMessage(
            `Windows: chemin direct pour le fond: ${backgroundUrl}`
          );
        } else {
          // Méthode pour Mac/Linux: utiliser convertFileSrc
          const bgFilePath = `${publicPath}/backgrounds/${data.background.file}`;
          addDebugMessage(`Mac/Linux: chemin image de fond: ${bgFilePath}`);

          const encodedBgPath = encodeURI(bgFilePath).replace(/#/g, "%23");
          backgroundUrl = convertFileSrc(encodedBgPath);
          addDebugMessage(
            `Mac/Linux: URL convertie pour le fond: ${backgroundUrl}`
          );
        }

        setBackgroundStyle({
          backgroundImage: `url(${backgroundUrl})`,
          backgroundSize: "cover",
          backgroundPosition: "center",
          backgroundRepeat: "no-repeat",
        });
      }
    } catch (error) {
      addDebugMessage(`ERREUR: Chargement de la configuration: ${error}`);
      if (error instanceof Error) {
        addDebugMessage(`ERREUR détails: ${error.message}`);
      }
    }
  }, [publicPath, config, addDebugMessage, isWindows]);

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
        let animationUrl;

        if (isWindows) {
          // Méthode pour Windows: chemin direct
          animationUrl = `./_up_/public/animations/${animation.file}`;
          addDebugMessage(
            `Windows: chemin direct pour l'animation: ${animationUrl}`
          );
        } else {
          // Méthode pour Mac/Linux: utiliser convertFileSrc
          const animPath = `${publicPath}/animations/${animation.file}`;
          addDebugMessage(`Mac/Linux: chemin animation: ${animPath}`);

          const encodedAnimPath = encodeURI(animPath).replace(/#/g, "%23");
          animationUrl = convertFileSrc(encodedAnimPath);
          addDebugMessage(
            `Mac/Linux: URL convertie pour l'animation: ${animationUrl}`
          );
        }

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
            <div>
              <span className={styles.platformIndicator}>
                {isWindows ? "Windows" : "Mac/Linux"}
              </span>
              <button
                onClick={() => setShowDebug(false)}
                className={styles.debugCloseButton}
              >
                Fermer
              </button>
            </div>
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
