<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="de">
<context>
    <name>MainWindow</name>
    <message><source>DubPlanetar — Soleil / Lune (CUDA) --- Version %1</source><translation>DubPlanetar — Sonne / Mond (CUDA) --- Version %1</translation></message>
    <message><source>GPU : %1</source><translation>GPU: %1</translation></message>
    <message><source>Cible</source><translation>Ziel</translation></message>
    <message><source>Objet à empiler :</source><translation>Zu stapelndes Objekt:</translation></message>
    <message><source>☀ Soleil</source><translation>☀ Sonne</translation></message>
    <message><source>☾ Lune</source><translation>☾ Mond</translation></message>
    <message><source>Vidéo source</source><translation>Quellvideo</translation></message>
    <message><source>Sélectionnez un AVI RAW SeeStar…</source><translation>SeeStar RAW AVI auswählen…</translation></message>
    <message><source>Parcourir…</source><translation>Durchsuchen…</translation></message>
    <message><source>Réglages</source><translation>Einstellungen</translation></message>
    <message><source>Frames conservées (moins = plus net)</source><translation>Behaltene Frames (weniger = schärfer)</translation></message>
    <message><source>Limite de frames (0 = toutes)</source><translation>Frame-Limit (0 = alle)</translation></message>
    <message><source>Toutes</source><translation>Alle</translation></message>
    <message><source>Debayer (SeeStar RAW)</source><translation>Debayer (SeeStar RAW)</translation></message>
    <message><source>Auto (recommandé)</source><translation>Auto (empfohlen)</translation></message>
    <message><source>Motif Bayer</source><translation>Bayer-Muster</translation></message>
    <message><source>Balance des blancs auto (corrige le voile vert)</source><translation>Auto-Weißabgleich (korrigiert Grünstich)</translation></message>
    <message><source>Recadrage auto sur le disque</source><translation>Auto-Zuschnitt auf Scheibe</translation></message>
    <message><source>Marge autour du disque</source><translation>Rand um die Scheibe</translation></message>
    <message><source>Drizzle x3 (super-résolution)</source><translation>Drizzle x3 (Superauflösung)</translation></message>
    <message><source>Tons (révèle les détails, évite les blancs brûlés)</source><translation>Töne (zeigt Details, vermeidet Ausreißer)</translation></message>
    <message><source>Aplatir le centre (anti-surexposition)</source><translation>Mitte abflachen (Anti-Überbelichtung)</translation></message>
    <message><source>Désactivé</source><translation>Deaktiviert</translation></message>
    <message><source>Gamma (&gt;1 révèle les détails sombres)</source><translation>Gamma (&gt;1 zeigt dunkle Details)</translation></message>
    <message><source>Netteté multi-échelle (ondelettes)</source><translation>Mehrskalige Schärfung (Wavelets)</translation></message>
    <message><source>Rayon (fin=1, large=3+)</source><translation>Radius (fein=1, breit=3+)</translation></message>
    <message><source>Compression Soleil (asinh, 0=off)</source><translation>Sonnenkompression (asinh, 0=aus)</translation></message>
    <message><source>Fond du ciel → noir (soustraction automatique)</source><translation>Himmelshintergrund → schwarz (auto Subtraktion)</translation></message>
    <message><source>Point noir supplémentaire</source><translation>Zusätzlicher Schwarzpunkt</translation></message>
    <message><source>Protéger les hautes lumières (ne pas brûler le centre)</source><translation>Lichter schützen (Mitte nicht ausbrennen)</translation></message>
    <message><source>Empiler</source><translation>Stapeln</translation></message>
    <message><source>Aperçu du résultat</source><translation>Ergebnisvorschau</translation></message>
    <message><source>Prêt.</source><translation>Bereit.</translation></message>
    <message><source>Choisir une vidéo AVI</source><translation>AVI-Video auswählen</translation></message>
    <message><source>Vidéos (*.avi *.AVI);;Tous les fichiers (*.*)</source><translation>Videos (*.avi *.AVI);;Alle Dateien (*.*)</translation></message>
    <message><source>Fichier manquant</source><translation>Datei fehlt</translation></message>
    <message><source>Sélectionnez un fichier AVI valide.</source><translation>Wählen Sie eine gültige AVI-Datei.</translation></message>
    <message><source>Traitement en cours…</source><translation>Verarbeitung läuft…</translation></message>
    <message><source>Temps écoulé : %1</source><translation>Verstrichene Zeit: %1</translation></message>
    <message><source>%1 min %2 s</source><translation>%1 Min %2 s</translation></message>
    <message><source>%1 s</source><translation>%1 s</translation></message>
    <message><source>, Bayer %1</source><translation>, Bayer %1</translation></message>
    <message><source>Terminé — %1/%2 frames%3 → %4</source><translation>Fertig — %1/%2 Frames%3 → %4</translation></message>
    <message><source>Temps total : %1</source><translation>Gesamtzeit: %1</translation></message>
    <message><source>Erreur</source><translation>Fehler</translation></message>
    <message><source>Arrêté après : %1</source><translation>Gestoppt nach: %1</translation></message>
    <message><source>Échec de l'empilement</source><translation>Stapeln fehlgeschlagen</translation></message>
</context>
<context>
    <name>Pipeline</name>
    <message><source>stage.read_avi</source><translation>AVI lesen</translation></message>
    <message><source>stage.read_frames</source><translation>Frames lesen…</translation></message>
    <message><source>stage.read_done</source><translation>Lesen abgeschlossen</translation></message>
    <message><source>stage.gpu_transfer</source><translation>GPU-Transfer</translation></message>
    <message><source>stage.sharpness</source><translation>Schärfeanalyse</translation></message>
    <message><source>stage.frame_select</source><translation>Frame-Auswahl</translation></message>
    <message><source>stage.align</source><translation>Ausrichtung</translation></message>
    <message><source>stage.crop</source><translation>Zuschnitt</translation></message>
    <message><source>stage.superres</source><translation>Superauflösungs-Stapelung</translation></message>
    <message><source>stage.bayer_detect</source><translation>Bayer-Mustererkennung</translation></message>
    <message><source>stage.done</source><translation>Fertig</translation></message>
    <message><source>gpu.available</source><translation>%1 — %2 GB VRAM frei</translation></message>
    <message><source>gpu.unavailable</source><translation>CUDA nicht verfügbar: %1</translation></message>
    <message><source>error.video_open</source><translation>Video kann nicht geöffnet werden: %1</translation></message>
    <message><source>error.no_frames</source><translation>Keine Frames aus dem Video gelesen.</translation></message>
    <message><source>error.unsupported_frame</source><translation>Nicht unterstütztes Frame-Format: %1</translation></message>
    <message><source>error.unknown_bayer</source><translation>Unbekanntes Bayer-Muster: %1</translation></message>
    <message><source>error.debayer_mono</source><translation>Debayer erwartet ein RAW-Einzelbild.</translation></message>
    <message><source>error.scale_min</source><translation>scale muss &gt;= 1 sein</translation></message>
</context>
</TS>
