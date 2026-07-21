# DubPlanetar (Nvidia CUDA)

📖 **[English version](README.md)**

**Empilement GPU (Nvidia CUDA) Soleil / Lune** pour vidéos AVI RAW capturées avec un appareil **SeeStar** (S50, S30, S30 Pro).

DubPlanetar transforme une séquence vidéo brute en une image finale super-résolue au format TIFF 16 bits, optimisée pour révéler les détails du disque solaire ou lunaire.

---

## Table des matières

- [Présentation](#présentation)
- [Fonctionnalités](#fonctionnalités)
- [Prérequis](#prérequis)
- [Installation (recommandée — double-clic)](#installation-recommandée--double-clic)
- [Installation (ligne de commande)](#installation-ligne-de-commande)
- [Lancement](#lancement)
- [Utilisation](#utilisation)
- [Profils Soleil et Lune](#profils-soleil-et-lune)
- [Paramètres détaillés](#paramètres-détaillés)
- [Pipeline technique](#pipeline-technique)
- [Format de sortie](#format-de-sortie)
- [Langues supportées](#langues-supportées)
- [Dépannage](#dépannage)
- [Crédits](#crédits)

---



## Présentation

L'astrophotographie planétaire avec un SeeStar produit des vidéos AVI en format RAW (capteur Bayer non dématriçé). Chaque frame individuelle est bruitée et affectée par les turbulences atmosphériques. L'empilement (stacking) consiste à :

1. Sélectionner les frames les plus nettes
2. Les aligner avec une précision sous-pixel
3. Les combiner pour améliorer le rapport signal/bruit
4. Appliquer un traitement d'image adapté à la cible (Soleil ou Lune)

DubPlanetar automatise l'ensemble de ce pipeline sur GPU NVIDIA via CUDA, avec une interface graphique intuitive.

**Résultat :** un fichier `*_stacked.tiff` recadré sur le disque, en super-résolution (×3 par défaut), prêt pour le post-traitement ou l'affichage.

---



## Fonctionnalités

- Interface graphique **PySide6** (Qt) avec aperçu du résultat
- Traitement **100 % GPU** (CuPy / CUDA) — score de netteté, alignement, empilement, debayer
- Profils prédéfinis **Soleil** et **Lune** avec réglages optimisés
- Détection automatique du motif Bayer (BGGR, GRBG, GBRG, RGGB)
- Recadrage automatique sur le disque avec marge réglable
- Super-résolution par **drizzle** (facteur ×3)
- Traitement tonal avancé : gamma, solar stretch, aplatissement du centre, netteté multi-échelle
- Interface multilingue : **français, anglais, espagnol, allemand**
- Sauvegarde automatique des préférences par profil

---



## Prérequis


| Composant     | Exigence                                                 |
| ------------- | -------------------------------------------------------- |
| Système       | Windows 10/11 ou Ubuntu 22.04+ (64 bits)                 |
| GPU           | NVIDIA avec support CUDA 12.x (testé sur RTX 3060 12 Go) |
| Pilotes       | Pilotes NVIDIA récents avec CUDA 12                      |
| Python        | 3.11 ou supérieur                                        |
| Appareil      | SeeStar S50, S30 ou S30 Pro                              |
| Format source | Fichiers AVI RAW (non compressés, capteur Bayer)         |


> **Note CUDA :** le projet utilise `cupy-cuda12x`. Si vous avez CUDA 11, installez plutôt `cupy-cuda11x` dans `requirements.txt`.

> **Environnement virtuel :** le dossier `.venv` est créé **sur la machine locale** et est exclu de Git (ne jamais le pousser sur GitHub). Ne copiez pas un `.venv` Windows vers Linux ni l’inverse — recréez-le toujours sur l’OS cible (`Scripts/` sous Windows, `bin/` sous Ubuntu).

---



## Installation (recommandée — double-clic)



### 1. Préparer l’ordinateur (une seule fois)

1. **Pilotes NVIDIA** récents (CUDA 12)
2. **Python 3.11 ou plus** :
   - Windows : [python.org/downloads](https://www.python.org/downloads/) — cochez **Add python.exe to PATH**
   - Ubuntu : en général déjà présent ; sinon `sudo apt install python3 python3-venv python3-pip`



### 2. Obtenir DubPlanetar

Téléchargez le projet (ZIP GitHub → Extraire) ou clonez le dépôt, puis ouvrez le dossier `dubplanetar`.



### 3. Installer (double-clic)

| Système | Fichier à double-cliquer |
| ------- | ------------------------ |
| **Ubuntu** | `Installer DubPlanetar` (fichier `.desktop`) |
| **Windows** | `Installer-DubPlanetar.bat` |

Attendez la fin du message « Installation terminée ».

> **Ubuntu — première fois :** le bureau peut demander de faire confiance au lanceur (*Faire confiance et lancer* / *Autoriser l’exécution*). C’est normal, une seule fois.



### 4. Lancer ensuite (double-clic)

| Système | Fichier à double-cliquer |
| ------- | ------------------------ |
| **Ubuntu** | `DubPlanetar` (fichier `.desktop`) |
| **Windows** | `Lancer-DubPlanetar.bat` (ou `Lancer-DubPlanetar.vbs` pour éviter la fenêtre noire) |

---



## Installation (ligne de commande)

Pour les utilisateurs à l’aise avec un terminal.



### 1. Cloner le dépôt

```bash
git clone https://github.com/Creations-Daniel-Dube/dubplanetar.git
cd dubplanetar
```



### 2. Lancer l’installateur (crée un `.venv` natif pour votre OS)

**Windows (PowerShell) :**

```powershell
.\install-dubplanetar.ps1
```

**Ubuntu :**

```bash
./install-dubplanetar.sh
```

Ou sur les deux OS :

```bash
python install_dubplanetar.py    # Windows
python3 install_dubplanetar.py   # Ubuntu
```

Cela crée `.venv`, installe les dépendances, installe le projet en mode editable et vérifie CUDA.



### 3. Compiler les traductions (si les fichiers `.qm` sont absents)

```bash
python scripts/compile_translations.py
```

> Les fichiers `.qm` compilés sont inclus dans le dépôt ; cette étape n'est nécessaire que si vous modifiez les fichiers de traduction `.ts`.



### 4. Vérifier CUDA

```bash
python -c "import cupy as cp; d=cp.cuda.Device(0); d.use(); print(cp.cuda.runtime.getDeviceProperties(0)['name'])"
```

Si cette commande affiche le nom de votre GPU, l'installation est réussie.

---



## Lancement



### Via double-clic (recommandé)

Voir [Installation (recommandée — double-clic)](#installation-recommandée--double-clic) — étape 4.



### Via le script de lancement (terminal)

**Windows :**

```powershell
.\launch-dubplanetar.ps1
```

**Ubuntu :**

```bash
./launch-dubplanetar.sh
```

Ou sur les deux OS :

```bash
python launch_dubplanetar.py    # Windows
python3 launch_dubplanetar.py   # Ubuntu
```



### Via Python (venv activé ou interpréteur du venv)

```bash
# Windows
.\.venv\Scripts\python.exe -m dub_planetar

# Ubuntu
.venv/bin/python -m dub_planetar
```



### Via la commande installée

```bash
dubplanetar
```

---



## Utilisation

1. **Lancer** DubPlanetar
2. **Choisir la cible** : ☀ Soleil ou ☾ Lune (charge le profil correspondant)
3. **Sélectionner** un fichier AVI RAW SeeStar via *Parcourir…*
4. **Ajuster** les paramètres si nécessaire (les valeurs par défaut du profil conviennent dans la plupart des cas)
5. Cliquer sur **Empiler**
6. Suivre la progression dans la barre et l'aperçu à droite
7. Le fichier `*_stacked.tiff` est créé **à côté de la vidéo source**

---



## Profils Soleil et Lune

Les profils préconfigurent les paramètres pour chaque cible. Vos réglages sont sauvegardés séparément par profil.


| Paramètre          | Profil Soleil | Profil Lune |
| ------------------ | ------------- | ----------- |
| Frames conservées  | 50 %          | 50 %        |
| Balance des blancs | Désactivée    | Activée     |
| Aplatir le centre  | 70 %          | Désactivé   |
| Debayer            | Activé        | Activé      |
| Recadrage auto     | Activé        | Activé      |
| Drizzle ×3         | Activé        | Activé      |


---



## Paramètres détaillés



### Sélection des frames

- **Frames conservées** : pourcentage des frames les plus nettes à garder (10–100 %). Moins = plus net, mais moins de signal.
- **Limite de frames** : nombre maximum de frames à lire (0 = toutes).



### Traitement RAW

- **Debayer** : convertit le capteur Bayer en couleur. Désactiver pour un empilement monochrome.
- **Motif Bayer** : `AUTO` (recommandé), ou forcer BGGR / GRBG / GBRG / RGGB.
- **Balance des blancs auto** : corrige le voile vert typique des capteurs SeeStar.



### Recadrage et super-résolution

- **Recadrage auto** : détecte le disque et recadre automatiquement.
- **Marge autour du disque** : espace supplémentaire autour du disque détecté (1–30 %).
- **Drizzle ×3** : empilement super-résolu (résolution finale ×3 en largeur et hauteur).



### Traitement tonal

- **Aplatir le centre** : réduit la surexposition au centre du Soleil (0 = désactivé).
- **Gamma** : révèle les détails sombres (> 1.0 éclaircit les ombres).
- **Netteté multi-échelle** : renforcement par ondelettes (0 = désactivé).
- **Compression Soleil (asinh)** : étirement adaptatif pour le disque solaire (0 = désactivé).
- **Fond du ciel → noir** : soustraction automatique du fond.
- **Point noir** : seuil bas pour le noir (en %).
- **Protéger les hautes lumières** : évite de brûler le centre du disque.

---



## Pipeline technique

Le traitement s'exécute entièrement sur GPU :

```
Vidéo AVI RAW
    │
    ▼
1. Lecture des frames (OpenCV)
    │
    ▼
2. Transfert GPU + score de netteté (variance du Laplacien)
    │
    ▼
3. Sélection des meilleures frames
    │
    ▼
4. Alignement sous-pixel (corrélation de phase, cuFFT)
    │
    ▼
5. Recadrage automatique sur le disque (Soleil/Lune)
    │
    ▼
6. Debayer BGGR dense (interpolation bilinéaire)
    │
    ▼
7. Empilement super-résolu (shift-and-add sous-pixel, drizzle ×3)
    │
    ▼
8. Balance des blancs + normalisation → TIFF 16 bits
```

---



## Format de sortie

- **Fichier** : `<nom_video>_stacked.tiff` (même dossier que la source)
- **Profondeur** : 16 bits par canal
- **Canaux** : RGB (debayer activé) ou monochrome
- **Résolution** : environ ×3 la taille du disque recadré (avec drizzle activé)
- **Espace colorimétrique** : linéaire, normalisé avec gamma appliqué

---



## Langues supportées

L'interface s'adapte automatiquement à la langue du système :


| Code | Langue   |
| ---- | -------- |
| `fr` | Français |
| `en` | Anglais  |
| `es` | Espagnol |
| `de` | Allemand |


Pour modifier les traductions, éditez les fichiers `.ts` dans `src/dub_planetar/translations/`, puis recompilez avec `python scripts/compile_translations.py`.

---



## Dépannage



### « GPU indisponible » au démarrage

- Vérifiez que vous avez une carte NVIDIA avec pilotes à jour
- Installez CUDA 12.x ou adaptez `requirements.txt` à votre version CUDA
- Testez : `python -c "import cupy; print(cupy.cuda.runtime.getDeviceCount())"`



### Erreur à l'import de CuPy

Assurez-vous que la variante CuPy correspond à votre CUDA :

```bash
# CUDA 12.x (par défaut)
pip install cupy-cuda12x[ctk]

# CUDA 11.x
pip install cupy-cuda11x[ctk]
```



### L'interface est en anglais malgré un système français

Les fichiers `.qm` compilés sont peut-être absents. Exécutez :

```bash
python scripts/compile_translations.py
```



### La vidéo ne s'ouvre pas

- Vérifiez que le fichier est un AVI RAW SeeStar (non compressé)
- Essayez de limiter le nombre de frames pour tester sur un extrait



### Résultat trop sombre ou trop clair

- Ajustez le **gamma** (1.2 par défaut)
- Pour le Soleil, activez **Aplatir le centre** (70 % dans le profil Soleil)
- Vérifiez le **point noir** et la protection des hautes lumières

---



## Crédits

**DubPlanetar** — Créations Daniel Dubé

Logiciel d'empilement GPU pour astrophotographie planétaire SeeStar.

---



## Licence

Tous droits réservés © Créations Daniel Dubé.