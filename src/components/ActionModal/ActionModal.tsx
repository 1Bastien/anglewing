import React, { useState, useEffect, useCallback } from "react";
import styles from "./ActionModal.module.css";
import { exit } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";

interface ActionModalProps {
  onClose: () => void;
}

interface Config {
  inactivity_timeout_in_minutes: number;
  background: {
    file: string;
  };
  security: {
    pin: string;
  };
  animations: Array<{
    id: number;
    title: string;
    file: string;
    playCount: number;
  }>;
}

const ActionModal: React.FC<ActionModalProps> = ({ onClose }) => {
  const [showPinInput, setShowPinInput] = useState(false);
  const [pin, setPin] = useState("");
  const [selectedAction, setSelectedAction] = useState<"close" | "shutdown" | null>(null);
  const [error, setError] = useState("");
  const [config, setConfig] = useState<Config | null>(null);

  const loadConfig = useCallback(async () => {
    try {
      const response = await fetch('../public/config.json');

      if (!response.ok) {
        const errorMsg = `Erreur HTTP ${response.status}: ${response.statusText}`;
        throw new Error(errorMsg);
      }

      const data = await response.json();
      setConfig(data);
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  }, []);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  const handleShutdown = async () => {
    try {
      await invoke('shutdown_system');
      setTimeout(() => {
        exit(0);
      }, 1000);
    } catch (error) {
      console.error("Failed to shutdown system:", error);
      setError("Erreur lors de l'extinction du système");
    }
  };

  const handlePinSubmit = async () => {
    if (!config || !config.security || !config.security.pin) {
      setError("Configuration non chargée ou invalide");
      return;
    }

    if (pin === config.security.pin) {
      if (selectedAction === "close") {
        try {
          await exit(0);
        } catch (error) {
          console.error("Failed to close application:", error);
          setError("Erreur lors de la fermeture de l'application");
        }
      } else if (selectedAction === "shutdown") {
        await handleShutdown();
      }
    } else {
      setError("Code PIN incorrect");
      setPin("");
    }
  };

  const handleActionClick = (action: "close" | "shutdown") => {
    setSelectedAction(action);
    setShowPinInput(true);
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        {!showPinInput ? (
          <>
            <h2>Que souhaitez-vous faire ?</h2>
            <div className={styles.buttonContainer}>
              <button onClick={() => handleActionClick("close")}>
                Fermer l'application
              </button>
              <button onClick={() => handleActionClick("shutdown")}>
                Éteindre l'ordinateur
              </button>
            </div>
          </>
        ) : (
          <>
            <h2>Entrez le code PIN</h2>
            <input
              type="password"
              value={pin}
              onChange={(e) => {
                setPin(e.target.value);
                setError("");
              }}
              maxLength={4}
              placeholder="****"
              className={styles.pinInput}
            />
            {error && <p className={styles.error}>{error}</p>}
            <div className={styles.buttonContainer}>
              <button onClick={handlePinSubmit}>Valider</button>
              <button onClick={() => {
                setShowPinInput(false);
                setPin("");
                setError("");
              }}>Retour</button>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default ActionModal; 