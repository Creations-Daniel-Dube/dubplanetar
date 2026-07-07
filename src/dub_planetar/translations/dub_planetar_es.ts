<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="es">
<context>
    <name>MainWindow</name>
    <message><source>DubPlanetar — Soleil / Lune (CUDA)</source><translation>DubPlanetar — Sol / Luna (CUDA)</translation></message>
    <message><source>GPU : %1</source><translation>GPU: %1</translation></message>
    <message><source>Cible</source><translation>Objetivo</translation></message>
    <message><source>Objet à empiler :</source><translation>Objeto a apilar:</translation></message>
    <message><source>☀ Soleil</source><translation>☀ Sol</translation></message>
    <message><source>☾ Lune</source><translation>☾ Luna</translation></message>
    <message><source>Vidéo source</source><translation>Vídeo de origen</translation></message>
    <message><source>Sélectionnez un AVI RAW SeeStar…</source><translation>Seleccione un AVI RAW SeeStar…</translation></message>
    <message><source>Parcourir…</source><translation>Examinar…</translation></message>
    <message><source>Réglages</source><translation>Ajustes</translation></message>
    <message><source>Frames conservées (moins = plus net)</source><translation>Fotogramas conservados (menos = más nítido)</translation></message>
    <message><source>Limite de frames (0 = toutes)</source><translation>Límite de fotogramas (0 = todos)</translation></message>
    <message><source>Toutes</source><translation>Todos</translation></message>
    <message><source>Debayer (SeeStar RAW)</source><translation>Debayer (SeeStar RAW)</translation></message>
    <message><source>Auto (recommandé)</source><translation>Auto (recomendado)</translation></message>
    <message><source>Motif Bayer</source><translation>Patrón Bayer</translation></message>
    <message><source>Balance des blancs auto (corrige le voile vert)</source><translation>Balance de blancos auto (corrige el velo verde)</translation></message>
    <message><source>Recadrage auto sur le disque</source><translation>Recorte auto en el disco</translation></message>
    <message><source>Marge autour du disque</source><translation>Margen alrededor del disco</translation></message>
    <message><source>Drizzle x3 (super-résolution)</source><translation>Drizzle x3 (super-resolución)</translation></message>
    <message><source>Tons (révèle les détails, évite les blancs brûlés)</source><translation>Tonos (revela detalles, evita blancos quemados)</translation></message>
    <message><source>Aplatir le centre (anti-surexposition)</source><translation>Aplanar el centro (anti-sobreexposición)</translation></message>
    <message><source>Désactivé</source><translation>Desactivado</translation></message>
    <message><source>Gamma (&gt;1 révèle les détails sombres)</source><translation>Gamma (&gt;1 revela detalles oscuros)</translation></message>
    <message><source>Netteté multi-échelle (ondelettes)</source><translation>Nitidez multiescala (wavelets)</translation></message>
    <message><source>Rayon (fin=1, large=3+)</source><translation>Radio (fino=1, amplio=3+)</translation></message>
    <message><source>Compression Soleil (asinh, 0=off)</source><translation>Compresión Sol (asinh, 0=off)</translation></message>
    <message><source>Fond du ciel → noir (soustraction automatique)</source><translation>Fondo del cielo → negro (sustracción automática)</translation></message>
    <message><source>Point noir supplémentaire</source><translation>Punto negro adicional</translation></message>
    <message><source>Protéger les hautes lumières (ne pas brûler le centre)</source><translation>Proteger luces altas (no quemar el centro)</translation></message>
    <message><source>Empiler</source><translation>Apilar</translation></message>
    <message><source>Aperçu du résultat</source><translation>Vista previa del resultado</translation></message>
    <message><source>Prêt.</source><translation>Listo.</translation></message>
    <message><source>Choisir une vidéo AVI</source><translation>Elegir un vídeo AVI</translation></message>
    <message><source>Vidéos (*.avi *.AVI);;Tous les fichiers (*.*)</source><translation>Vídeos (*.avi *.AVI);;Todos los archivos (*.*)</translation></message>
    <message><source>Fichier manquant</source><translation>Archivo faltante</translation></message>
    <message><source>Sélectionnez un fichier AVI valide.</source><translation>Seleccione un archivo AVI válido.</translation></message>
    <message><source>Traitement en cours…</source><translation>Procesando…</translation></message>
    <message><source>Temps écoulé : %1</source><translation>Tiempo transcurrido: %1</translation></message>
    <message><source>%1 min %2 s</source><translation>%1 min %2 s</translation></message>
    <message><source>%1 s</source><translation>%1 s</translation></message>
    <message><source>, Bayer %1</source><translation>, Bayer %1</translation></message>
    <message><source>Terminé — %1/%2 frames%3 → %4</source><translation>Terminado — %1/%2 fotogramas%3 → %4</translation></message>
    <message><source>Temps total : %1</source><translation>Tiempo total: %1</translation></message>
    <message><source>Erreur</source><translation>Error</translation></message>
    <message><source>Arrêté après : %1</source><translation>Detenido después de: %1</translation></message>
    <message><source>Échec de l'empilement</source><translation>Error al apilar</translation></message>
</context>
<context>
    <name>Pipeline</name>
    <message><source>stage.read_avi</source><translation>Lectura AVI</translation></message>
    <message><source>stage.read_frames</source><translation>Leyendo fotogramas…</translation></message>
    <message><source>stage.read_done</source><translation>Lectura terminada</translation></message>
    <message><source>stage.gpu_transfer</source><translation>Transferencia GPU</translation></message>
    <message><source>stage.sharpness</source><translation>Análisis de nitidez</translation></message>
    <message><source>stage.frame_select</source><translation>Selección de fotogramas</translation></message>
    <message><source>stage.align</source><translation>Alineación</translation></message>
    <message><source>stage.crop</source><translation>Recorte</translation></message>
    <message><source>stage.superres</source><translation>Apilado super-resolución</translation></message>
    <message><source>stage.bayer_detect</source><translation>Detección patrón Bayer</translation></message>
    <message><source>stage.done</source><translation>Terminado</translation></message>
    <message><source>gpu.available</source><translation>%1 — %2 GB VRAM libre</translation></message>
    <message><source>gpu.unavailable</source><translation>CUDA no disponible: %1</translation></message>
    <message><source>error.video_open</source><translation>No se puede abrir el vídeo: %1</translation></message>
    <message><source>error.no_frames</source><translation>No se leyó ningún fotograma del vídeo.</translation></message>
    <message><source>error.unsupported_frame</source><translation>Formato de fotograma no compatible: %1</translation></message>
    <message><source>error.unknown_bayer</source><translation>Patrón Bayer desconocido: %1</translation></message>
    <message><source>error.debayer_mono</source><translation>El debayer espera una imagen RAW mono-plano.</translation></message>
    <message><source>error.scale_min</source><translation>scale debe ser &gt;= 1</translation></message>
</context>
</TS>
