#!/usr/bin/env bash
#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Version  ----------------->   00.08.250     *
#* Dernières Modifications -->   2026-09-11    *
#***********************************************
# Installe DubPlanetar sous Ubuntu / Linux : crée un .venv natif (bin/), jamais versionné.
set -euo pipefail
cd "$(dirname "$0")"
exec python3 -u install_dubplanetar.py "$@"
