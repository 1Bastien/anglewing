import React, { useEffect, useState } from "react";
import styles from "./Home.module.css";
import Button from "../Button/Button";
import CloseButton from "../CloseButton/CloseButton";
import AnimationPlayer from "../AnimationPlayer/AnimationPlayer";

const Home: React.FC = () => {
  const [backgroundStyle, setBackgroundStyle] = useState({});
  const [selectedAnimation, setSelectedAnimation] = useState<string | null>(
    null
  );

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

  const handleAnimationSelect = (animationNumber: number) => {
    setSelectedAnimation(`/animations/Animation_${animationNumber}.mp4`);
  };

  const closeAnimationPlayer = () => {
    setSelectedAnimation(null);
  };

  return (
    <div className={styles.container}>
      <CloseButton />
      <div className={styles.backgroundImage} style={backgroundStyle}></div>
      <div className={styles.buttonsContainer}>
        <Button text="Animation 1" onClick={() => handleAnimationSelect(1)} />
        <Button text="Animation 2" onClick={() => handleAnimationSelect(2)} />
        <Button text="Animation 3" onClick={() => handleAnimationSelect(3)} />
      </div>

      {selectedAnimation && (
        <AnimationPlayer
          animationUrl={selectedAnimation}
          onClose={closeAnimationPlayer}
        />
      )}
    </div>
  );
};

export default Home;
