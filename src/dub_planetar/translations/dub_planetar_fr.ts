<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="fr">
<context>
    <name>MainWindow</name>
    <message><source>DubPlanetar — Soleil / Lune (CUDA)</source><translation>DubPlanetar — Soleil / Lune (CUDA)</translation></message>
    <message><source>GPU : %1</source><translation>GPU : %1</translation></message>
    <message><source>Cible</source><translation>Cible</translation></message>
    <message><source>Objet à empiler :</source><translation>Objet à empiler :</translation></message>
    <message><source>☀ Soleil</source><translation>☀ Soleil</translation></message>
    <message><source>☾ Lune</source><translation>☾ Lune</translation></message>
    <message><source>Vidéo source</source><translation>Vidéo source</translation></message>
    <message><source>Sélectionnez un AVI RAW SeeStar…</source><translation>Sélectionnez un AVI RAW SeeStar…</translation></message>
    <message><source>Parcourir…</source><translation>Parcourir…</translation></message>
    <message><source>Réglages</source><translation>Réglages</translation></message>
    <message><source>Frames conservées (moins = plus net)</source><translation>Frames conservées (moins = plus net)</translation></message>
    <message><source>Limite de frames (0 = toutes)</source><translation>Limite de frames (0 = toutes)</translation></message>
    <message><source>Toutes</source><translation>Toutes</translation></message>
    <message><source>Debayer (SeeStar RAW)</source><translation>Debayer (SeeStar RAW)</translation></message>
    <message><source>Auto (recommandé)</source><translation>Auto (recommandé)</translation></message>
    <message><source>Motif Bayer</source><translation>Motif Bayer</translation></message>
    <message><source>Balance des blancs auto (corrige le voile vert)</source><translation>Balance des blancs auto (corrige le voile vert)</translation></message>
    <message><source>Recadrage auto sur le disque</source><translation>Recadrage auto sur le disque</translation></message>
    <message><source>Marge autour du disque</source><translation>Marge autour du disque</translation></message>
    <message><source>Drizzle x3 (super-résolution)</source><translation>Drizzle x3 (super-résolution)</translation></message>
    <message><source>Tons (révèle les détails, évite les blancs brûlés)</source><translation>Tons (révèle les détails, évite les blancs brûlés)</translation></message>
    <message><source>Aplatir le centre (anti-surexposition)</source><translation>Aplatir le centre (anti-surexposition)</translation></message>
    <message><source>Désactivé</source><translation>Désactivé</translation></message>
    <message><source>Gamma (&gt;1 révèle les détails sombres)</source><translation>Gamma (&gt;1 révèle les détails sombres)</translation></message>
    <message><source>Netteté multi-échelle (ondelettes)</source><translation>Netteté multi-échelle (ondelettes)</translation></message>
    <message><source>Rayon (fin=1, large=3+)</source><translation>Rayon (fin=1, large=3+)</translation></message>
    <message><source>Compression Soleil (asinh, 0=off)</source><translation>Compression Soleil (asinh, 0=off)</translation></message>
    <message><source>Fond du ciel → noir (soustraction automatique)</source><translation>Fond du ciel → noir (soustraction automatique)</translation></message>
    <message><source>Point noir supplémentaire</source><translation>Point noir supplémentaire</translation></message>
    <message><source>Protéger les hautes lumières (ne pas brûler le centre)</source><translation>Protéger les hautes lumières (ne pas brûler le centre)</translation></message>
    <message><source>Empiler</source><translation>Empiler</translation></message>
    <message><source>Aperçu du résultat</source><translation>Aperçu du résultat</translation></message>
    <message><source>Prêt.</source><translation>Prêt.</translation></message>
    <message><source>Choisir une vidéo AVI</source><translation>Choisir une vidéo AVI</translation></message>
    <message><source>Vidéos (*.avi *.AVI);;Tous les fichiers (*.*)</source><translation>Vidéos (*.avi *.AVI);;Tous les fichiers (*.*)</translation></message>
    <message><source>Fichier manquant</source><translation>Fichier manquant</translation></message>
    <message><source>Sélectionnez un fichier AVI valide.</source><translation>Sélectionnez un fichier AVI valide.</translation></message>
    <message><source>Traitement en cours…</source><translation>Traitement en cours…</translation></message>
    <message><source>Temps écoulé : %1</source><translation>Temps écoulé : %1</translation></message>
    <message><source>%1 min %2 s</source><translation>%1 min %2 s</translation></message>
    <message><source>%1 s</source><translation>%1 s</translation></message>
    <message><source>, Bayer %1</source><translation>, Bayer %1</translation></message>
    <message><source>Terminé — %1/%2 frames%3 → %4</source><translation>Terminé — %1/%2 frames%3 → %4</translation></message>
    <message><source>Temps total : %1</source><translation>Temps total : %1</translation></message>
    <message><source>Erreur</source><translation>Erreur</translation></message>
    <message><source>Arrêté après : %1</source><translation>Arrêté après : %1</translation></message>
    <message><source>Échec de l'empilement</source><translation>Échec de l'empilement</translation></message>
</context>
<context>
    <name>Pipeline</name>
    <message><source>stage.read_avi</source><translation>Lecture AVI</translation></message>
    <message><source>stage.read_frames</source><translation>Lecture des frames…</translation></message>
    <message><source>stage.read_done</source><translation>Lecture terminée</translation></message>
    <message><source>stage.gpu_transfer</source><translation>Transfert GPU</translation></message>
    <message><source>stage.sharpness</source><translation>Analyse netteté</translation></message>
    <message><source>stage.frame_select</source><translation>Sélection des frames</translation></message>
    <message><source>stage.align</source><translation>Alignement</translation></message>
    <message><source>stage.crop</source><translation>Recadrage</translation></message>
    <message><source>stage.superres</source><translation>Empilement super-résolu</translation></message>
    <message><source>stage.bayer_detect</source><translation>Détection motif Bayer</translation></message>
    <message><source>stage.done</source><translation>Terminé</translation></message>
    <message><source>gpu.available</source><translation>%1 — %2 Go VRAM libre</translation></message>
    <message><source>gpu.unavailable</source><translation>CUDA indisponible : %1</translation></message>
    <message><source>error.video_open</source><translation>Impossible d'ouvrir la vidéo : %1</translation></message>
    <message><source>error.no_frames</source><translation>Aucune frame lue dans la vidéo.</translation></message>
    <message><source>error.unsupported_frame</source><translation>Format de frame non supporté : %1</translation></message>
    <message><source>error.unknown_bayer</source><translation>Motif Bayer inconnu : %1</translation></message>
    <message><source>error.debayer_mono</source><translation>Le debayer attend une image RAW mono-plan.</translation></message>
    <message><source>error.scale_min</source><translation>scale doit être &gt;= 1</translation></message>
</context>
</TS>
