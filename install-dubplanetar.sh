#!/usr/bin/env bash
# Installe DubPlanetar sous Ubuntu / Linux : crée un .venv natif (bin/), jamais versionné.
set -euo pipefail
cd "$(dirname "$0")"
exec python3 install_dubplanetar.py "$@"
