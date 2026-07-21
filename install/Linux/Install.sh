#!/usr/bin/env bash
# DubPlanetar — décompresse le pack puis lance l'installation (Linux).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Double-clic sans terminal : rouvrir dans un terminal visible.
if [[ ! -t 1 ]]; then
  title="DubPlanetar — installation"
  cmd="bash \"$SCRIPT_DIR/$(basename "$0")\""
  for term in gnome-terminal konsole xfce4-terminal mate-terminal x-terminal-emulator xterm; do
    if command -v "$term" >/dev/null 2>&1; then
      case "$term" in
        gnome-terminal) exec gnome-terminal --title="$title" -- bash -lc "$cmd" ;;
        konsole)        exec konsole --title "$title" -e bash -lc "$cmd" ;;
        *)              exec "$term" -e bash -lc "$cmd" ;;
      esac
    fi
  done
fi

ARCHIVE="DubPlanetar-0.6.75.tar.gz"
DEST="DubPlanetar"

echo "========================================================"
echo "  DubPlanetar — installation (Linux)"
echo "========================================================"
echo

if [[ ! -f "$ARCHIVE" ]]; then
  echo "ERREUR : archive introuvable : $ARCHIVE"
  echo "Placez Install.sh et $ARCHIVE dans le même dossier."
  echo
  read -r -p "Appuyez sur Entrée pour fermer… " || true
  exit 1
fi

if [[ -e "$DEST" ]]; then
  echo "[1/3] Le dossier $DEST existe déjà — suppression..."
  rm -rf "$DEST"
else
  echo "[1/3] Prêt à extraire."
fi

echo "[2/3] Extraction de $ARCHIVE ..."
tar -xzf "$ARCHIVE"
echo "      Extraction terminée."

if [[ ! -f "$DEST/install-dubplanetar.sh" ]]; then
  echo "ERREUR : $DEST/install-dubplanetar.sh introuvable après extraction."
  read -r -p "Appuyez sur Entrée pour fermer… " || true
  exit 1
fi

chmod +x "$DEST/install-dubplanetar.sh" "$DEST/launch-dubplanetar.sh" \
  "$DEST/scripts/run_desktop_action.sh" 2>/dev/null || true

echo "[3/3] Installation des dépendances (venv, pip, raccourci Bureau)..."
echo "      Suivez la progression ci-dessous…"
echo

set +e
"$DEST/install-dubplanetar.sh"
status=$?
set -e

echo
if [[ "$status" -ne 0 ]]; then
  echo "========================================================"
  echo "  ÉCHEC DE L'INSTALLATION (code $status)"
  echo "========================================================"
else
  echo "========================================================"
  echo "  INSTALLATION TERMINÉE"
  echo "  - Raccourci : Bureau / DubPlanetar"
  echo "  - L'application devrait s'être ouverte"
  echo "========================================================"
fi
echo
read -r -p "Appuyez sur Entrée pour fermer… " || true
exit "$status"
