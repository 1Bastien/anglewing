import React, { useEffect, useState, useCallback, useRef } from "react";
import styles from "./Home.module.css";
import Button from "../Button/Button";
import CloseButton from "../CloseButton/CloseButton";
import AnimationPlayer from "../AnimationPlayer/AnimationPlayer";
import { invoke } from "@tauri-apps/api/core";

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
  const [selectedAnimation, setSelectedAnimation] = useState<AnimationConfig | null>(null);
  const [config, setConfig] = useState<Config | null>(null);
  const inactivityTimerRef = useRef<number | null>(null);

  // Commented out dynamic path retrieval
  // const getPublicPath = async () => { ... }

  const loadConfig = useCallback(async () => {
    const response = await fetch('../public/config.json');

    if (!response.ok) {
      const errorMsg = `Erreur HTTP ${response.status}: ${response.statusText}`;
      throw new Error(errorMsg);
    }

    const data = await response.json();

    if (JSON.stringify(data) === JSON.stringify(config)) {
      return;
    }

    setConfig(data);

    if (data.background) {
      const bgFilePath = `../public/backgrounds/${data.background.file}`;
      setBackgroundStyle({
        backgroundImage: `url(${bgFilePath})`,
        backgroundSize: "cover",
        backgroundPosition: "center",
        backgroundRepeat: "no-repeat",
      });
    }
  }, [config]);

  const resetInactivityTimer = useCallback(() => {
    if (inactivityTimerRef.current !== null) {
      window.clearTimeout(inactivityTimerRef.current);
    }

    invoke("reset_inactivity_timer").catch(() => {});

    inactivityTimerRef.current = window.setTimeout(async () => {
      if (!selectedAnimation) {
        try {
          await invoke("put_system_to_sleep");
        } catch (error) {
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
        const animPath = `../public/animations/${animation.file}`;
        setSelectedAnimation({
          ...animation,
          file: animPath,
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
            onClick={() => {
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
            closeAnimationPlayer();
          }}
        />
      )}
    </div>
  );
};

export default Home;
