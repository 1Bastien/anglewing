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
  const [useFallbackMethod, setUseFallbackMethod] = useState(false);

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
      let configPath;
      let response;

      if (!useFallbackMethod) {
        // Méthode 1: Utiliser convertFileSrc avec le chemin complet
        const filePath = `${publicPath}/config.json`;
        addDebugMessage(`Méthode 1 - Chemin config.json: ${filePath}`);

        const encodedPath = encodeURI(filePath).replace(/#/g, "%23");
        configPath = convertFileSrc(encodedPath);
        addDebugMessage(`Méthode 1 - URL convertie: ${configPath}`);

        addDebugMessage("Tentative de chargement avec chemin complet...");
        try {
          response = await fetch(configPath);
          addDebugMessage("Fetch config.json réussi avec méthode 1!");
        } catch (fetchError) {
          addDebugMessage(`Erreur fetch méthode 1: ${fetchError}`);
          throw fetchError;
        }
      } else {
        // Méthode 2: Essayer avec un chemin relatif simple
        // Extraire le dernier segment du chemin (où _up_ devrait être)
        const segments = publicPath.split("/");
        const upFolder = segments.findIndex((segment) => segment === "_up_");

        if (upFolder >= 0) {
          const relativePath = segments.slice(upFolder).join("/");
          addDebugMessage(
            `Méthode 2 - Chemin relatif: ${relativePath}/config.json`
          );

          configPath = `./${relativePath}/config.json`;
          addDebugMessage(`Méthode 2 - URL relative: ${configPath}`);

          addDebugMessage("Tentative de chargement avec chemin relatif...");
          try {
            response = await fetch(configPath);
            addDebugMessage("Fetch config.json réussi avec méthode 2!");
          } catch (fetchError) {
            addDebugMessage(`Erreur fetch méthode 2: ${fetchError}`);
            throw fetchError;
          }
        } else {
          // Méthode 3: Essayer avec un chemin ultra simplifié
          configPath = `./_up_/public/config.json`;
          addDebugMessage(`Méthode 3 - URL simplifiée: ${configPath}`);

          addDebugMessage("Tentative de chargement avec chemin simplifié...");
          try {
            response = await fetch(configPath);
            addDebugMessage("Fetch config.json réussi avec méthode 3!");
          } catch (fetchError) {
            addDebugMessage(`Erreur fetch méthode 3: ${fetchError}`);
            throw fetchError;
          }
        }
      }

      if (!response.ok) {
        const errMsg = `Erreur HTTP ${response.status}: ${response.statusText}`;
        addDebugMessage(`ERREUR HTTP: ${errMsg}`);

        if (response.status === 403 && !useFallbackMethod) {
          addDebugMessage(
            "Erreur 403 détectée - Passage à la méthode alternative"
          );
          setUseFallbackMethod(true);
          return; // Sortir pour que le useEffect relance loadConfig
        }

        throw new Error(errMsg);
      }

      const data = await response.json();
      addDebugMessage("Configuration JSON parsée avec succès");

      if (JSON.stringify(data) === JSON.stringify(config)) {
        return;
      }

      setConfig(data);

      if (data.background) {
        let backgroundUrl;

        if (!useFallbackMethod) {
          // Méthode standard
          const bgFilePath = `${publicPath}/backgrounds/${data.background.file}`;
          addDebugMessage(`Chemin image de fond: ${bgFilePath}`);

          const encodedBgPath = encodeURI(bgFilePath).replace(/#/g, "%23");
          backgroundUrl = convertFileSrc(encodedBgPath);
        } else {
          // Méthode alternative
          const segments = publicPath.split("/");
          const upFolder = segments.findIndex((segment) => segment === "_up_");

          if (upFolder >= 0) {
            const relativePath = segments.slice(upFolder).join("/");
            backgroundUrl = `./${relativePath}/backgrounds/${data.background.file}`;
          } else {
            backgroundUrl = `./_up_/public/backgrounds/${data.background.file}`;
          }
        }

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
      if (error instanceof Error) {
        addDebugMessage(`ERREUR détails: ${error.message}`);

        // Si on n'a pas encore essayé la méthode alternative, on bascule
        if (!useFallbackMethod) {
          addDebugMessage(
            "Echec avec la méthode standard - Passage à la méthode alternative"
          );
          setUseFallbackMethod(true);
        }
      }
    }
  }, [publicPath, config, addDebugMessage, useFallbackMethod]);

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
  }, [loadConfig, publicPath, useFallbackMethod]);

  const handleAnimationSelect = (animationId: number) => {
    if (config && publicPath) {
      const animation = config.animations.find((a) => a.id === animationId);
      if (animation) {
        let animationUrl;

        if (!useFallbackMethod) {
          // Méthode standard
          const animPath = `${publicPath}/animations/${animation.file}`;
          addDebugMessage(`Chemin animation: ${animPath}`);

          const encodedAnimPath = encodeURI(animPath).replace(/#/g, "%23");
          animationUrl = convertFileSrc(encodedAnimPath);
        } else {
          // Méthode alternative
          const segments = publicPath.split("/");
          const upFolder = segments.findIndex((segment) => segment === "_up_");

          if (upFolder >= 0) {
            const relativePath = segments.slice(upFolder).join("/");
            animationUrl = `./${relativePath}/animations/${animation.file}`;
          } else {
            animationUrl = `./_up_/public/animations/${animation.file}`;
          }
        }

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
            <div>
              <button
                onClick={() => setUseFallbackMethod(!useFallbackMethod)}
                className={styles.debugActionButton}
              >
                {useFallbackMethod ? "Méthode Alt" : "Méthode Std"}
              </button>
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
