import React, { useState } from "react";
import styles from "./ActionModal.module.css";
import { exit } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";

interface ActionModalProps {
  onClose: () => void;
}

const ActionModal: React.FC<ActionModalProps> = ({ onClose }) => {
  const [showPinInput, setShowPinInput] = useState(false);
  const [pin, setPin] = useState("");
  const [selectedAction, setSelectedAction] = useState<"close" | "shutdown" | null>(null);
  const [error, setError] = useState("");

  const correctPin = "1234";

  const handleShutdown = async () => {
    try {
      // Lancer la commande d'extinction d'abord
      await invoke('shutdown_system');
      // Puis fermer l'application
      setTimeout(() => {
        exit(0);
      }, 1000);
    } catch (error) {
      console.error("Failed to shutdown system:", error);
      setError("Erreur lors de l'extinction du système");
    }
  };

  const handlePinSubmit = async () => {
    if (pin === correctPin) {
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