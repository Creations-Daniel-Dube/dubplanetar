# dubPlanetar — Planètes à l'échelle

**Version 0.4.2000 BETA**

Application de bureau pour visualiser les planètes, la Lune et le Soleil telles qu'elles apparaîtraient dans un télescope, avec les vraies tailles angulaires. Les planètes et la Lune montrent aussi leur phase, à l'instant choisi. Le Soleil suit l'heure de l'ordinateur.

Le dépôt GitHub : [Creations-Daniel-Dube/dubplanetar](https://github.com/Creations-Daniel-Dube/dubplanetar)

---

## Manuel de l'utilisateur (français)

### Démarrage

Lancez le programme **Planètes à l'échelle**. Une fenêtre s'ouvre : menu à gauche, ciel nocturne et vue à droite. Au premier démarrage, la vue **Planètes à l'échelle** est affichée avec la date et l'heure actuelles (arrondies à la minute).

### Menu (colonne de gauche)

Quatre entrées mènent aux parties du logiciel :

| Bouton | Rôle |
|--------|------|
| **Date et heure** | Choisir l'instant de l'observation (planètes et Lune). |
| **Planètes à l'échelle** | Mercure à Neptune sur une même échelle en secondes d'arc. |
| **Lune** | La Lune seule, en grand sur tout l'écran. |
| **Soleil** | Le Soleil maintenant, à l'heure de l'ordinateur. |

### Date et heure

C'est l'**horloge commune** des planètes et de la Lune. Tout ce que vous réglez ici s'applique en même temps à ces deux vues. L'onglet **Soleil** ne la suit pas : il reste à la date et à l'heure actuelles de l'ordinateur.

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

**Liste — Vu depuis**

- Par défaut : **la Terre**. Mercure à Neptune gardent leurs places.
- Une autre planète : la vue est celle qu'on aurait depuis cette planète. Elle quitte l'écran et **la Terre** occupe sa case, avec le diamètre, la phase et l'orientation recalculés.
- Le même choix vaut pour l'onglet **Soleil**. La Lune, elle, reste vue de la Terre.

**Survol de la souris**

Passez la souris sur un disque pour un encadré détaillé : distance en unités astronomiques, diamètre, % éclairé, angle de phase, latitude sous l'observateur, orientation du pôle, angle du limbe éclairé.

**Double-clic — satellites**

Seulement quand **Vu depuis** est la Terre. Un double-clic sur Mars, Jupiter, Saturne, Uranus ou Neptune ouvre une fenêtre. Mercure et Vénus n'ont pas de satellites : le double-clic ne fait rien.

La planète est dessinée comme dans la grille (phase, carte, anneaux de Saturne, axe bleu). Autour, les satellites les plus lumineux sont à leur place réelle, vue de la Terre :

| Planète | Satellites |
|---------|------------|
| Mars | Phobos, Déimos |
| Jupiter | Io, Europe, Ganymède, Callisto |
| Saturne | Téthys, Dioné, Rhéa, Titan, Japet |
| Uranus | Miranda, Ariel, Umbriel, Titania, Obéron |
| Neptune | Triton (les autres sont trop faibles) |

Le champ de départ contient l'orbite la plus large : les lunes sont alors des points, classés par magnitude. La molette zoome sans changer les écarts ni les diamètres. Dès qu'un satellite fait au moins trois pixels (surtout les lunes de Jupiter, et Titan), il devient un petit disque avec sa phase. Une lune derrière la planète, ou dans son ombre, disparaît. Sur Jupiter et Saturne, l'ombre de la lune est dessinée sur le disque quand elle tombe du côté visible. Au-delà de la magnitude 16, le point n'est plus tracé.

La fenêtre suit la date et l'animation. La case **Nord vers le bas** est celle de l'écran. La croix ou Échap ferment la fenêtre. Changer **Vu depuis** la ferme aussi.

### Lune

Même date et heure que les planètes, mais **un seul disque**, aussi grand que le cadre le permet. Une Lune de **35′** le remplit ; plus elle est loin, plus le disque rétrécit (super-Lune près du périgée, plus petite à l’apogée).

**En-tête** : titre, date, orientation du Nord (même case à cocher que pour les planètes).

**Sous le disque** : nom, diamètre en **minutes d'arc** (′), pourcentage éclairé.

**Relief (optionnel)** : si une carte de hauteur `moon_bump` est présente (voir ci-dessous), les crêtes et pentes sont un peu plus claires ou plus sombres selon l'angle du Soleil — surtout près du **terminateur**. Ce n'est pas une simulation d'ombres portées au fond des cratères.

**Survol** : distance en kilomètres, diamètre en minutes d'arc, % éclairé, phase, latitude sous-terrestre.

### Soleil

Cet onglet **ne suit pas** la date des autres vues. Il affiche toujours le Soleil pour **l'heure actuelle de l'ordinateur**. Il n'a pas de bandeau d'animation.

À chaque lancement, le logiciel télécharge l'image **SDO/HMI continuum** (quick-look NASA) et n'en garde que le disque : la légende en bas de la photo est retirée. Le Soleil n'a pas de phase : le cercle est toujours entier. Sa taille à l'écran suit la **vraie distance entre l'observateur et le Soleil**, avec le Soleil vu de **Mercure** comme référence : ce disque remplit le cadre, et les autres planètes sont plus petites dans le même rapport. Le texte sous le disque garde le diamètre réel en minutes d'arc. Depuis la Terre, il est un peu plus grand début janvier, au périhélie, qu'au début juillet, à l'aphélie (environ 3 %). Depuis Neptune le disque devient très petit. La photo reste celle vue de la Terre : seule la taille change.

La liste **Vu depuis** est la même que dans **Planètes à l'échelle**.

**En-tête** : titre, heure de l'ordinateur, origine de l'image, case **Nord vers le bas** (rotation de 180°, les textes restent lisibles).

**Sous le disque** : nom, diamètre en **minutes d'arc** (′), distance en unités astronomiques.

Si le réseau manque, la dernière image enregistrée est réaffichée. Sans aucune image en réserve, un disque uni de secours prend la place, à la même échelle.

### Animation

En bas des vues **Planètes à l'échelle** et **Lune**, le bandeau **Animation** fait défiler la date commune.

- **Lecture** avance, **Reculer** recule, **Arrêt** fige l'instant dès le premier clic. Le sens en cours est en vert. Le premier pas part tout de suite, puis un pas toutes les 0,7 s, une fois l'image affichée.
- **1 fois** avance d'un seul saut, puis s'arrête. Un clic, un pas.
- Pendant la préparation, **Génération en cours...** s'affiche au centre. L'image précédente reste visible, et Arrêt, le menu et la date restent utilisables.
- **Saut** : un nombre et une unité. Jours de 1 à 365, mois de 1 à 12, années de 1 à 10.
- L'heure et la minute ne changent pas. Un 31 ramené dans un mois plus court tombe sur le dernier jour de ce mois.
- La lecture s'arrête à l'an 1000 et à l'an 2500.
- Les onglets **Date et heure** et **Soleil** n'ont pas de bandeau. La date montre l'instant pendant la lecture. Le Soleil continue d'afficher l'heure de l'ordinateur. Changer d'onglet ne l'arrête pas.

### Images optionnelles

Le logiciel **n'inclut pas** de photos NASA dans le dépôt. Vous pouvez enrichir l'affichage en plaçant des fichiers image dans le dossier **`assets/textures`** à côté du programme (ou du dossier de travail habituel).

| Fichier | Corps |
|---------|--------|
| `mercury.png`, `venus.png`, `mars.png` | Planètes |
| `jupiter.png`, `saturn.png`, `uranus.png`, `neptune.png` | Planètes |
| `earth.png` | Terre (même cadrage que les autres planètes) |
| `moon.png` | Lune (carte équatoriale, nord en haut) |

Formats acceptés : **PNG** ou **JPEG**. Les planètes utilisent la longitude 0° **à gauche** de l'image ; la Lune place la longitude 0° **au centre** (face principale au milieu).

**Relief lunaire (optionnel)** : `moon_bump.png` (ou `.jpg`), en **niveaux de gris**, **plus clair = plus haut**, même cadrage que `moon.png`. Sans ce fichier, la Lune reste lisse à part le terminateur.

Après avoir ajouté ou remplacé une image, **changez de minute ou cliquez dans la vue** pour forcer un nouvel affichage.

### Limites connues (version bêta)

- Un seul instant à la fois (pas de comparaison entre plusieurs dates).
- Pas de comètes ni d'astéroïdes. Les satellites montrés sont les plus lumineux de chaque planète ; Neptune n'affiche que Triton.
- Leur place est celle d'un atlas amateur (environ une minute d'arc), pas une éphéméride de sonde.
- Les photos des planètes et de la Lune sont des cartes fixes : pas de météo, pas de taches en évolution sur Jupiter, etc.
- L'image du Soleil est le dernier continuum SDO/HMI disponible au lancement (souvent différé de quelques minutes), pas une reconstitution pour une date passée. Depuis une autre planète, cette photo ne tourne pas pour montrer l'hémisphère solaire qui lui fait face.
- L'éclairage du relief lunaire accentue les pentes ; il ne calcule pas les ombres profondes dans les cratères.
- Calculs pensés pour les usages amateurs (années 1000–2500) ; ce n'est pas un outil professionnel de mission spatiale.

### Aide rapide

1. Réglez **Date et heure** (ou **Maintenant**).
2. Ouvrez **Planètes à l'échelle** pour comparer les tailles et phases.
3. Ouvrez **Lune** pour le détail lunaire.
4. Ouvrez **Soleil** pour le disque du moment, à l'heure de l'ordinateur.
5. Cochez ou décochez **Nord vers le bas** selon votre habitude d'observation.
6. Dans **Animation**, choisissez le saut, puis **Lecture**, **Reculer** ou **1 fois**.

---

## User manual (English)

### Getting started

Open **Planètes à l'échelle** (*Planets at scale*). The window has a left-hand menu and a main sky view. On first launch, the **Planètes à l'échelle** tab is shown with the current local date and time (rounded to the minute).

### Left menu

| Button | Purpose |
|--------|---------|
| **Date et heure** | Set the observation time (planets and Moon). |
| **Planètes à l'échelle** | Mercury through Neptune on one arcsecond scale. |
| **Lune** | The Moon alone, large on screen. |
| **Soleil** | The Sun right now, on the computer's clock. |

### Date and time

This is the **shared clock** for the planets and the Moon. Whatever you set here applies at once to those two views. The **Soleil** tab ignores it and stays on the computer's current date and time.

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

**List — Vu depuis** (*Seen from*)

- Default: **Earth**. Mercury through Neptune keep their usual places.
- Another planet: the view is from that planet. It leaves the screen and **Earth** takes its cell, with diameter, phase, and orientation recomputed.
- The same choice applies to the **Soleil** tab. The Moon stays as seen from Earth.

**Mouse hover**

Tooltip on each disk: distance (AU), diameter, illuminated %, phase angle, sub-observer latitude, pole position angle, bright-limb angle.

**Double-click — satellites**

Only when **Vu depuis** is Earth. Double-click Mars, Jupiter, Saturn, Uranus, or Neptune to open a window. Mercury and Venus have no moons, so the double-click does nothing.

The planet is drawn as on the grid (phase, map, Saturn's rings, blue axis). Around it, the brightest moons sit at their true places as seen from Earth:

| Planet | Satellites |
|--------|------------|
| Mars | Phobos, Deimos |
| Jupiter | Io, Europa, Ganymede, Callisto |
| Saturn | Tethys, Dione, Rhea, Titan, Iapetus |
| Uranus | Miranda, Ariel, Umbriel, Titania, Oberon |
| Neptune | Triton (the others are too faint) |

The opening field contains the widest orbit, so the moons are points ranked by magnitude. The scroll wheel zooms without changing separations or diameters. Once a moon is at least three pixels across (mainly Jupiter's moons, and Titan), it is drawn as a small disk with its phase. A moon behind the planet, or in its shadow, disappears. On Jupiter and Saturn the moon's shadow is drawn on the disk when it falls on the visible face. Fainter than magnitude 16, the point is not drawn.

The window follows the date and the animation. It uses the same **Nord vers le bas** checkbox as the screen. The close box or Escape dismisses it. Changing **Vu depuis** closes it too.

### Moon

Same date and time as the planets, with a **single disk** as large as the frame allows. A Moon of **35′** fills it; a more distant Moon is drawn smaller (a supermoon near perigee, a smaller one at apogee).

**Header**: title, date, same north checkbox.

**Labels**: name, diameter in **arcminutes** (′), illuminated %.

**Optional relief**: with a grayscale height map `moon_bump` (see below), slopes catch sunlight near the **terminator** more strongly. This is slope shading, not ray-traced crater shadows.

**Hover**: distance (km), diameter (′), illuminated %, phase, sub-Earth latitude.

### Sun

This tab **does not follow** the date used by the other views. It always shows the Sun for the **computer's current time**. It has no animation bar.

On each launch the app downloads the NASA **SDO/HMI continuum** quick-look and keeps only the disk: the caption along the bottom of the photo is removed. The Sun has no phase, so the circle is always full. Its on-screen size follows the **true distance from the chosen planet to the Sun**, with the Sun as seen from **Mercury** as the reference: that disk fills the frame, and the other planets are smaller in the same ratio. The caption under the disk keeps the real diameter in arcminutes. From Earth it is slightly larger in early January, at perihelion, than in early July, at aphelion (about 3%). From Neptune the disk becomes very small. The photo stays the one taken from Earth: only the size changes.

The **Vu depuis** list is the same one as on **Planètes à l'échelle**.

**Header**: title, computer time, image source, and the **Nord vers le bas** checkbox (a 180° turn; labels stay upright).

**Under the disk**: name, diameter in **arcminutes** (′), distance in astronomical units.

If the network is down, the last saved image is shown again. With no saved image, a plain fallback disk is drawn at the same scale.

### Animation

At the bottom of **Planètes à l'échelle** and **Lune**, the **Animation** bar steps the shared date.

- **Lecture** plays forward, **Reculer** plays backward, **Arrêt** stops on the first click. The active direction is green. The first step is immediate, then one step every 0.7 s once the picture is on screen.
- **1 fois** moves forward by a single step, then stops. One click, one step.
- While a picture is prepared, **Génération en cours...** appears in the center. The previous picture stays up, and Stop, the menu, and the date stay usable.
- **Saut**: a count and a unit. Days 1–365, months 1–12, years 1–10.
- Hour and minute stay put. A day that does not exist in the target month (31 January plus one month) lands on the last day of that month.
- Playback stops at year 1000 and year 2500.
- **Date et heure** and **Soleil** have no bar. The date tab shows the moving instant. The Sun tab keeps showing the computer's clock. Switching tabs does not stop playback.

### Optional images

The app **does not ship** planetary photo maps. Place files in **`assets/textures`** next to the program (or your usual working folder).

| File | Body |
|------|------|
| `mercury.png` … `neptune.png` | Planets |
| `earth.png` | Earth (same framing as the other planets) |
| `moon.png` | Moon (equirectangular, north up) |

**PNG** or **JPEG**. Planets: longitude 0° at the **left** edge of the map. Moon: longitude 0° at the **center** (near side centered).

**Optional Moon relief**: `moon_bump.png` or `.jpg`, grayscale, **brighter = higher**, same framing as `moon.png`. Without it, the Moon uses smooth lighting only.

After adding or changing a file, **change the time by one minute or click the view** to refresh.

### Known limits (beta)

- One instant at a time (no multi-date comparison).
- No comets or asteroids. The moons shown are the brightest of each planet; Neptune shows only Triton.
- Their places are amateur-atlas accuracy (about one arcminute), not a spacecraft ephemeris.
- Planet and Moon textures are static; no weather or evolving surface features.
- The Sun picture is the latest SDO/HMI continuum available at launch (often a few minutes behind), not a reconstruction for a past date. From another planet, that photo does not rotate to the solar hemisphere facing that planet.
- Lunar bump shading exaggerates slopes; it does not model deep crater shadowing.
- Suited to amateur use (years 1000–2500), not professional mission planning.

### Quick workflow

1. Set **Date et heure** (or **Maintenant**).
2. Open **Planètes à l'échelle** to compare sizes and phases.
3. Open **Lune** for the Moon.
4. Open **Soleil** for the Sun as it is now, on the computer's clock.
5. Toggle **Nord vers le bas** to match your observing habit.
6. In **Animation**, set the step, then press **Lecture**, **Reculer**, or **1 fois**.

---

## Licence

Voir le dépôt pour les conditions de distribution. Les cartes d'images que vous ajoutez localement restent sous **votre** responsabilité (droits d'auteur et sources).

*Creations-Daniel-Dube*
