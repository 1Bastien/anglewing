import React from "react";
import styles from "./CloseButton.module.css";
import { exit } from "@tauri-apps/plugin-process";

const CloseButton: React.FC = () => {
  const handleClose = async () => {
    try {
      await exit(0);
    } catch (error) {
      console.error("Failed to close application:", error);
    }
  };

  return (
    <button
      className={styles.closeButton}
      onClick={handleClose}
      aria-label="Close application"
    >
      <span className={styles.hidden}>Close</span>
    </button>
  );
};

export default CloseButton;
