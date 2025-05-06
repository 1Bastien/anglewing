GUIDE D'UTILISATION - CONFIGURATION

FICHIER CONFIG.JSON
-------------------

Le fichier config.json permet de configurer les paramètres suivants:

1. DÉLAI D'INACTIVITÉ
   - La valeur "inactivity_timeout_in_minutes" définit le délai d'inactivité en minutes.
   - Exemple: "inactivity_timeout_in_minutes": 10

2. ARRIÈRE-PLAN
   - La section "background" permet de définir l'image d'arrière-plan.
   - Spécifiez le nom de fichier dans "file".
   - Exemple: "background": { "file": "background.png" }
   - Les fichiers d'arrière-plan doivent être placés dans le dossier "/backgrounds".

3. ANIMATIONS
   - La section "animations" contient la liste des animations à utiliser.
   - Chaque animation comprend:
     * "id": identifiant unique de l'animation
     * "title": titre de l'animation
     * "file": nom du fichier de l'animation
     * "playCount": nombre de fois que l'animation sera jouée
   - Exemple:
     {
       "id": 1,
       "title": "Animation 1",
       "file": "Animation_1.mp4",
       "playCount": 3
     }
   - Les fichiers d'animation doivent être placés dans le dossier "/animations".

COMMENT MODIFIER LES ANIMATIONS ET ARRIÈRE-PLANS
------------------------------------------------
1. Pour AJOUTER DE NOUVELLES ANIMATIONS:
   - Placez les fichiers d'animation (.mp4) dans le dossier "/animations"
   - Ajoutez une nouvelle entrée dans la liste "animations" du fichier config.json

2. Pour CHANGER L'ARRIÈRE-PLAN:
   - Placez le fichier d'arrière-plan dans le dossier "/backgrounds"
   - Modifiez la valeur "file" dans la section "background" du config.json

NOTE IMPORTANTE: Vous pouvez stocker des animations et des arrière-plans supplémentaires dans les dossiers respectifs sans les utiliser immédiatement. Il suffit de modifier le fichier config.json pour les activer quand vous le souhaitez. L'important est que les noms des fichiers dans config.json correspondent exactement aux fichiers présents dans les dossiers. 