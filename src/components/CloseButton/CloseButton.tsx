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
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M6.7 5.3a1 1 0 0 0-1.4 1.4L10.6 12l-5.3 5.3a1 1 0 0 0 1.4 1.4L12 13.4l5.3 5.3a1 1 0 0 0 1.4-1.4L13.4 12l5.3-5.3a1 1 0 0 0-1.4-1.4L12 10.6 6.7 5.3z"
            fill="white"
          />
        </svg>
      </button>
      {showModal && <ActionModal onClose={() => setShowModal(false)} />}
    </>
  );
};

export default CloseButton;
