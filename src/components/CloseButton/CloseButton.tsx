import React, { useState } from "react";
import styles from "./CloseButton.module.css";
import ActionModal from "../ActionModal/ActionModal";

const CloseButton: React.FC = () => {
  const [showModal, setShowModal] = useState(false);

  return (
    <>
      <button
        className={styles.closeButton}
        onClick={() => setShowModal(true)}
        aria-label="Close application"
      >
        <span className={styles.hidden}>Close</span>
      </button>
      {showModal && <ActionModal onClose={() => setShowModal(false)} />}
    </>
  );
};

export default CloseButton;
