#!/usr/bin/env bash
#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Version  ----------------->   00.08.250     *
#* Dernières Modifications -->   2026-09-11    *
#***********************************************
# Lance DubPlanetar (Ubuntu / Linux) via autodétection du .venv local.
set -euo pipefail
cd "$(dirname "$0")"
exec python3 launch_dubplanetar.py "$@"
