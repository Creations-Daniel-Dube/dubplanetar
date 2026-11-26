# dubPlanetar — Planètes à l'échelle

**Version 0.1.500 BETA**

Application de bureau pour visualiser les planètes et la Lune telles qu'elles apparaîtraient dans un télescope, à un instant donné, avec les vraies tailles angulaires et les phases d'éclairage.

Le dépôt GitHub : [Creations-Daniel-Dube/dubplanetar](https://github.com/Creations-Daniel-Dube/dubplanetar)

---

## Manuel de l'utilisateur (français)

### Démarrage

Lancez le programme **Planètes à l'échelle**. Une fenêtre s'ouvre : menu à gauche, ciel nocturne et vue à droite. Au premier démarrage, la vue **Planètes à l'échelle** est affichée avec la date et l'heure actuelles (arrondies à la minute).

### Menu (colonne de gauche)

Trois entrées mènent aux parties du logiciel :

| Bouton | Rôle |
|--------|------|
| **Date et heure** | Choisir l'instant de l'observation (partagé par toutes les vues). |
| **Planètes à l'échelle** | Mercure à Neptune sur une même échelle en secondes d'arc. |
| **Lune** | La Lune seule, en grand sur tout l'écran. |

### Date et heure

C'est l'**horloge commune** du logiciel. Tout ce que vous réglez ici s'applique en même temps aux planètes et à la Lune.

- **Année** : de 1000 à 2500.
- **Mois** : liste en français.
- **Jour**, **heure** (0–23), **minute** (0–59).
- **Maintenant** : revient à la date et à l'heure locales du moment (minute tronquée).
- En bas : l'instant en **heure locale** (en vert) et l'équivalent **UTC**.

Changez l'année, le mois ou le jour : les jours invalides (par exemple 31 février) sont ajustés automatiquement.

### Planètes à l'échelle

Cette vue imite une page de revue d'astronomie : **toutes les planètes du Système solaire (Mercure à Neptune) au même instant**, avec une **échelle commune en secondes d'arc** («).

**En-tête**

- Titre, date et heure locales.
- Sous-titre selon l'orientation du Nord (voir ci-dessous).
- À droite : barre d'**échelle** graduée en secondes d'arc (par pas de 10″, jusqu'à 60″ ou moins si la fenêtre est étroite).

**Disposition**

- Ligne du haut : Mercure et Mars.
- Centre : Vénus (la plus grande sur l'écran).
- Ligne du bas : Jupiter, Saturne, Uranus, Neptune.

**Sous chaque disque**

- Nom de la planète ;
- **Diamètre** en secondes d'arc ;
- **Pourcentage éclairé** (0 % = noir total, 100 % = disque entièrement au Soleil).

**Rendu visuel**

- Phase réaliste : côté nuit noir, côté jour éclairé par le Soleil (modèle de type « Lambert »).
- Trait bleu : **axe du pôle nord** de la planète.
- **Saturne** : anneaux inclus lorsque l'inclinaison le permet ; ombre portée simplifiée sur les anneaux à certains angles.
- Sans photo personnelle : couleur unie typique de la planète.
- Avec photo (voir « Images optionnelles ») : carte en projection équatoriale, nord en haut sur la carte.

**Case à cocher — Nord vers le bas**

- Décochée (par défaut) : **le Nord est vers le haut** (convention carte / photographie).
- Cochée : comme dans un télescope astronomique inversé, **le Nord est vers le bas** à l'écran ; les textes restent lisibles.

**Survol de la souris**

Passez la souris sur un disque pour un encadré détaillé : distance en unités astronomiques, diamètre, % éclairé, angle de phase, latitude sous-terrestre, orientation du pôle, angle du limbe éclairé.

### Lune

Même date et heure que les planètes, mais **un seul disque** occupant le plus grand espace possible.

**En-tête** : titre, date, orientation du Nord (même case à cocher que pour les planètes).

**Sous le disque** : nom, diamètre en **minutes d'arc** (′), pourcentage éclairé.

**Relief (optionnel)** : si une carte de hauteur `moon_bump` est présente (voir ci-dessous), les crêtes et pentes sont un peu plus claires ou plus sombres selon l'angle du Soleil — surtout près du **terminateur**. Ce n'est pas une simulation d'ombres portées au fond des cratères.

**Survol** : distance en kilomètres, diamètre en minutes d'arc, % éclairé, phase, latitude sous-terrestre.

### Animation

En bas des vues **Planètes à l'échelle** et **Lune**, le bandeau **Animation** fait défiler la date commune.

- **Lecture** avance, **Reculer** recule, **Arrêt** fige l'instant. Le sens en cours est en vert. Le premier pas part tout de suite, puis un pas toutes les 0,7 s.
- **Saut** : un nombre et une unité. Jours de 1 à 365, mois de 1 à 12, années de 1 à 10.
- L'heure et la minute ne changent pas. Un 31 ramené dans un mois plus court tombe sur le dernier jour de ce mois.
- La lecture s'arrête à l'an 1000 et à l'an 2500.
- L'onglet **Date et heure** n'a pas de bandeau, mais il montre l'instant pendant la lecture. Changer d'onglet ne l'arrête pas.

### Images optionnelles

Le logiciel **n'inclut pas** de photos NASA dans le dépôt. Vous pouvez enrichir l'affichage en plaçant des fichiers image dans le dossier **`assets/textures`** à côté du programme (ou du dossier de travail habituel).

| Fichier | Corps |
|---------|--------|
| `mercury.png`, `venus.png`, `mars.png` | Planètes |
| `jupiter.png`, `saturn.png`, `uranus.png`, `neptune.png` | Planètes |
| `moon.png` | Lune (carte équatoriale, nord en haut) |

Formats acceptés : **PNG** ou **JPEG**. Les planètes utilisent la longitude 0° **à gauche** de l'image ; la Lune place la longitude 0° **au centre** (face principale au milieu).

**Relief lunaire (optionnel)** : `moon_bump.png` (ou `.jpg`), en **niveaux de gris**, **plus clair = plus haut**, même cadrage que `moon.png`. Sans ce fichier, la Lune reste lisse à part le terminateur.

Après avoir ajouté ou remplacé une image, **changez de minute ou cliquez dans la vue** pour forcer un nouvel affichage.

### Limites connues (version bêta)

- Un seul instant à la fois (pas de comparaison entre plusieurs dates).
- Pas de satellites, comètes ni astéroïdes.
- Les photos sont des cartes fixes : pas de météo, pas de taches en évolution sur Jupiter, etc.
- L'éclairage du relief lunaire accentue les pentes ; il ne calcule pas les ombres profondes dans les cratères.
- Calculs pensés pour les usages amateurs (années 1000–2500) ; ce n'est pas un outil professionnel de mission spatiale.

### Aide rapide

1. Réglez **Date et heure** (ou **Maintenant**).
2. Ouvrez **Planètes à l'échelle** pour comparer les tailles et phases.
3. Ouvrez **Lune** pour le détail lunaire.
4. Cochez ou décochez **Nord vers le bas** selon votre habitude d'observation.
5. Dans **Animation**, choisissez le saut, puis **Lecture** ou **Reculer**.

---

## User manual (English)

### Getting started

Open **Planètes à l'échelle** (*Planets at scale*). The window has a left-hand menu and a main sky view. On first launch, the **Planètes à l'échelle** tab is shown with the current local date and time (rounded to the minute).

### Left menu

| Button | Purpose |
|--------|---------|
| **Date et heure** | Set the observation time (shared by all views). |
| **Planètes à l'échelle** | Mercury through Neptune on one arcsecond scale. |
| **Lune** | The Moon alone, large on screen. |

### Date and time

This is the **shared clock** for the app. Whatever you set here applies at once to the planets view and the Moon view.

- **Year**: 1000–2500.
- **Month**: French names.
- **Day**, **hour** (0–23), **minute** (0–59).
- **Maintenant** (*Now*): reset to current local time (minute truncated).
- Bottom line: **local time** (green) and **UTC**.

Invalid calendar dates (e.g. 31 February) are clamped automatically.

### Planets at scale

All **major planets (Mercury to Neptune) at one instant**, on a **common scale in arcseconds** (″), similar to a magazine star chart.

**Header**

- Title, local date and time.
- Subtitle for north orientation (below).
- **Scale bar** on the right in arcseconds (10″ steps, up to 60″ or less on narrow windows).

**Layout**

- Top row: Mercury and Mars.
- Middle: Venus (largest on screen).
- Bottom row: Jupiter, Saturn, Uranus, Neptune.

**Under each disk**

- Planet name;
- **Diameter** in arcseconds;
- **Illuminated fraction** in percent.

**Appearance**

- Realistic terminator: night side black, day side sunlit.
- Blue line: planet **north pole axis**.
- **Saturn**: rings when geometry allows; simplified ring shadow in some configurations.
- Without custom images: solid representative colors.
- With custom maps: equirectangular texture, north up on the map.

**Checkbox — Nord vers le bas** (*North down*)

- Unchecked (default): **north at the top** (chart / photo convention).
- Checked: **north at the bottom**, as in many astronomical telescopes; labels stay upright.

**Mouse hover**

Tooltip on each disk: distance (AU), diameter, illuminated %, phase angle, sub-Earth latitude, pole position angle, bright-limb angle.

### Moon

Same date and time as the planets, with a **single large disk**.

**Header**: title, date, same north checkbox.

**Labels**: name, diameter in **arcminutes** (′), illuminated %.

**Optional relief**: with a grayscale height map `moon_bump` (see below), slopes catch sunlight near the **terminator** more strongly. This is slope shading, not ray-traced crater shadows.

**Hover**: distance (km), diameter (′), illuminated %, phase, sub-Earth latitude.

### Animation

At the bottom of **Planètes à l'échelle** and **Lune**, the **Animation** bar steps the shared date.

- **Lecture** plays forward, **Reculer** plays backward, **Arrêt** stops. The active direction is green. The first step is immediate, then one step every 0.7 s.
- **Saut**: a count and a unit. Days 1–365, months 1–12, years 1–10.
- Hour and minute stay put. A day that does not exist in the target month (31 January plus one month) lands on the last day of that month.
- Playback stops at year 1000 and year 2500.
- **Date et heure** has no bar, but it shows the moving instant. Switching tabs does not stop playback.

### Optional images

The app **does not ship** planetary photo maps. Place files in **`assets/textures`** next to the program (or your usual working folder).

| File | Body |
|------|------|
| `mercury.png` … `neptune.png` | Planets |
| `moon.png` | Moon (equirectangular, north up) |

**PNG** or **JPEG**. Planets: longitude 0° at the **left** edge of the map. Moon: longitude 0° at the **center** (near side centered).

**Optional Moon relief**: `moon_bump.png` or `.jpg`, grayscale, **brighter = higher**, same framing as `moon.png`. Without it, the Moon uses smooth lighting only.

After adding or changing a file, **change the time by one minute or click the view** to refresh.

### Known limits (beta)

- One instant at a time (no multi-date comparison).
- No moons, comets, or asteroids.
- Textures are static; no weather or evolving surface features.
- Lunar bump shading exaggerates slopes; it does not model deep crater shadowing.
- Suited to amateur use (years 1000–2500), not professional mission planning.

### Quick workflow

1. Set **Date et heure** (or **Maintenant**).
2. Open **Planètes à l'échelle** to compare sizes and phases.
3. Open **Lune** for the Moon.
4. Toggle **Nord vers le bas** to match your observing habit.
5. In **Animation**, set the step, then press **Lecture** or **Reculer**.

---

## Licence

Voir le dépôt pour les conditions de distribution. Les cartes d'images que vous ajoutez localement restent sous **votre** responsabilité (droits d'auteur et sources).

*Creations-Daniel-Dube*