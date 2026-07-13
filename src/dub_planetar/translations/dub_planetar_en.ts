<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="en">
<context>
    <name>MainWindow</name>
    <message><source>DubPlanetar — Soleil / Lune (CUDA) --- Version %1</source><translation>DubPlanetar — Sun / Moon (CUDA) --- Version %1</translation></message>
    <message><source>GPU : %1</source><translation>GPU: %1</translation></message>
    <message><source>Cible</source><translation>Target</translation></message>
    <message><source>Objet à empiler :</source><translation>Object to stack:</translation></message>
    <message><source>☀ Soleil</source><translation>☀ Sun</translation></message>
    <message><source>☾ Lune</source><translation>☾ Moon</translation></message>
    <message><source>Vidéo source</source><translation>Source video</translation></message>
    <message><source>Sélectionnez un AVI RAW SeeStar…</source><translation>Select a SeeStar RAW AVI…</translation></message>
    <message><source>Parcourir…</source><translation>Browse…</translation></message>
    <message><source>Réglages</source><translation>Settings</translation></message>
    <message><source>Frames conservées (moins = plus net)</source><translation>Frames kept (fewer = sharper)</translation></message>
    <message><source>Limite de frames (0 = toutes)</source><translation>Frame limit (0 = all)</translation></message>
    <message><source>Toutes</source><translation>All</translation></message>
    <message><source>Debayer (SeeStar RAW)</source><translation>Debayer (SeeStar RAW)</translation></message>
    <message><source>Auto (recommandé)</source><translation>Auto (recommended)</translation></message>
    <message><source>Motif Bayer</source><translation>Bayer pattern</translation></message>
    <message><source>Balance des blancs auto (corrige le voile vert)</source><translation>Auto white balance (fixes green cast)</translation></message>
    <message><source>Recadrage auto sur le disque</source><translation>Auto crop on disc</translation></message>
    <message><source>Marge autour du disque</source><translation>Margin around disc</translation></message>
    <message><source>Drizzle x3 (super-résolution)</source><translation>Drizzle x3 (super-resolution)</translation></message>
    <message><source>Tons (révèle les détails, évite les blancs brûlés)</source><translation>Tone (reveals detail, avoids blown highlights)</translation></message>
    <message><source>Aplatir le centre (anti-surexposition)</source><translation>Flatten center (anti-overexposure)</translation></message>
    <message><source>Désactivé</source><translation>Disabled</translation></message>
    <message><source>Gamma (&gt;1 révèle les détails sombres)</source><translation>Gamma (&gt;1 reveals dark detail)</translation></message>
    <message><source>Netteté multi-échelle (ondelettes)</source><translation>Multi-scale sharpening (wavelets)</translation></message>
    <message><source>Rayon (fin=1, large=3+)</source><translation>Radius (fine=1, wide=3+)</translation></message>
    <message><source>Compression Soleil (asinh, 0=off)</source><translation>Sun compression (asinh, 0=off)</translation></message>
    <message><source>Fond du ciel → noir (soustraction automatique)</source><translation>Sky background → black (auto subtraction)</translation></message>
    <message><source>Point noir supplémentaire</source><translation>Extra black point</translation></message>
    <message><source>Protéger les hautes lumières (ne pas brûler le centre)</source><translation>Protect highlights (don't burn the center)</translation></message>
    <message><source>Empiler</source><translation>Stack</translation></message>
    <message><source>Aperçu du résultat</source><translation>Result preview</translation></message>
    <message><source>Prêt.</source><translation>Ready.</translation></message>
    <message><source>Choisir une vidéo AVI</source><translation>Choose an AVI video</translation></message>
    <message><source>Vidéos (*.avi *.AVI);;Tous les fichiers (*.*)</source><translation>Videos (*.avi *.AVI);;All files (*.*)</translation></message>
    <message><source>Fichier manquant</source><translation>Missing file</translation></message>
    <message><source>Sélectionnez un fichier AVI valide.</source><translation>Select a valid AVI file.</translation></message>
    <message><source>Traitement en cours…</source><translation>Processing…</translation></message>
    <message><source>Temps écoulé : %1</source><translation>Elapsed time: %1</translation></message>
    <message><source>%1 min %2 s</source><translation>%1 min %2 s</translation></message>
    <message><source>%1 s</source><translation>%1 s</translation></message>
    <message><source>, Bayer %1</source><translation>, Bayer %1</translation></message>
    <message><source>Terminé — %1/%2 frames%3 → %4</source><translation>Done — %1/%2 frames%3 → %4</translation></message>
    <message><source>Temps total : %1</source><translation>Total time: %1</translation></message>
    <message><source>Erreur</source><translation>Error</translation></message>
    <message><source>Arrêté après : %1</source><translation>Stopped after: %1</translation></message>
    <message><source>Échec de l'empilement</source><translation>Stacking failed</translation></message>
</context>
<context>
    <name>Pipeline</name>
    <message><source>stage.read_avi</source><translation>Reading AVI</translation></message>
    <message><source>stage.read_frames</source><translation>Reading frames…</translation></message>
    <message><source>stage.read_done</source><translation>Read complete</translation></message>
    <message><source>stage.gpu_transfer</source><translation>GPU transfer</translation></message>
    <message><source>stage.sharpness</source><translation>Sharpness analysis</translation></message>
    <message><source>stage.frame_select</source><translation>Frame selection</translation></message>
    <message><source>stage.align</source><translation>Alignment</translation></message>
    <message><source>stage.crop</source><translation>Cropping</translation></message>
    <message><source>stage.superres</source><translation>Super-resolution stacking</translation></message>
    <message><source>stage.bayer_detect</source><translation>Bayer pattern detection</translation></message>
    <message><source>stage.done</source><translation>Done</translation></message>
    <message><source>gpu.available</source><translation>%1 — %2 GB VRAM free</translation></message>
    <message><source>gpu.unavailable</source><translation>CUDA unavailable: %1</translation></message>
    <message><source>error.video_open</source><translation>Cannot open video: %1</translation></message>
    <message><source>error.no_frames</source><translation>No frames read from the video.</translation></message>
    <message><source>error.unsupported_frame</source><translation>Unsupported frame format: %1</translation></message>
    <message><source>error.unknown_bayer</source><translation>Unknown Bayer pattern: %1</translation></message>
    <message><source>error.debayer_mono</source><translation>Debayer expects a single-plane RAW image.</translation></message>
    <message><source>error.scale_min</source><translation>scale must be &gt;= 1</translation></message>
</context>
</TS>
