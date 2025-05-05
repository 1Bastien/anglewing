import React, { useEffect, useState } from "react";
import styles from "./Home.module.css";
import Button from "../Button/Button";
import CloseButton from "../CloseButton/CloseButton";

const Home: React.FC = () => {
  const [backgroundStyle, setBackgroundStyle] = useState({});

  useEffect(() => {
    // Utiliser l'image depuis le dossier public/backgrounds
    // L'admin pourra simplement remplacer cette image sans rebuild
    const backgroundImageUrl = `/backgrounds/background.png`;
    setBackgroundStyle({
      backgroundImage: `url(${backgroundImageUrl})`,
      backgroundSize: "cover",
      backgroundPosition: "center",
      backgroundRepeat: "no-repeat",
    });
  }, []);

  return (
    <div className={styles.container}>
      <CloseButton />
      <div className={styles.backgroundImage} style={backgroundStyle}></div>
      <div className={styles.buttonsContainer}>
        <Button text="Animation 1" />
        <Button text="Animation 2" />
        <Button text="Animation 3" />
      </div>
    </div>
  );
};

export default Home;
