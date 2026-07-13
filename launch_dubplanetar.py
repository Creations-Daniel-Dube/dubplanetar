#!/usr/bin/env python3
#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-12    *
#***********************************************
"""Lance DubPlanetar via le Python du .venv natif (Windows ou Linux)."""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "src"))

from dub_planetar.platform_env import (  # noqa: E402
    install_script_command,
    is_windows,
    os_label,
    venv_python,
)


def main() -> int:
    python = venv_python(gui=True, root=ROOT)
    if not python.is_file():
        # Sous Windows, pythonw peut manquer si seul python.exe existe.
        python = venv_python(gui=False, root=ROOT)
    if not python.is_file():
        print("")
        print(f"DubPlanetar — environnement non installé ({os_label()}).")
        print(f"Exécutez d'abord : {install_script_command(ROOT)}")
        print("")
        if is_windows() and sys.stdin.isatty():
            try:
                input("Appuyez sur Entrée pour fermer")
            except EOFError:
                pass
        return 1

    os.chdir(ROOT)
    argv = [str(python), "-m", "dub_planetar", *sys.argv[1:]]
    if is_windows():
        return subprocess.call(argv)
    os.execv(str(python), argv)
    return 0  # pragma: no cover


if __name__ == "__main__":
    raise SystemExit(main())
