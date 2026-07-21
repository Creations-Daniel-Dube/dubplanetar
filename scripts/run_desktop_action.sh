#!/usr/bin/env bash
# Résout le dossier projet à partir du chemin/%k d'un fichier .desktop, puis
# exécute install ou launch. Utilisé par les lanceurs double-clic Ubuntu.
set -euo pipefail

usage() {
  echo "Usage: $0 <install|launch> <chemin-du-fichier.desktop>" >&2
  exit 2
}

[[ $# -ge 2 ]] || usage
action="$1"
desktop_ref="$2"

# file:// URI → chemin local (décode %20, etc.)
if [[ "$desktop_ref" == file://* ]]; then
  desktop_ref="${desktop_ref#file://}"
  if command -v python3 >/dev/null 2>&1; then
    desktop_ref="$(python3 -c "import sys, urllib.parse; print(urllib.parse.unquote(sys.argv[1]))" "$desktop_ref")"
  fi
fi

if [[ ! -e "$desktop_ref" ]]; then
  echo "ERREUR : fichier .desktop introuvable : $desktop_ref" >&2
  exit 1
fi

root="$(cd "$(dirname "$desktop_ref")" && pwd)"
cd "$root"

case "$action" in
  install)
    ./install-dubplanetar.sh
    status=$?
    echo
    echo "Code de sortie : $status"
    read -r -p "Appuyez sur Entrée pour fermer… " || true
    exit "$status"
    ;;
  launch)
    exec ./launch-dubplanetar.sh
    ;;
  *)
    usage
    ;;
esac
