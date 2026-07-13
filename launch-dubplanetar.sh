#!/usr/bin/env bash
# Lance DubPlanetar (Ubuntu / Linux) via autodétection du .venv local.
set -euo pipefail
cd "$(dirname "$0")"
exec python3 launch_dubplanetar.py "$@"
