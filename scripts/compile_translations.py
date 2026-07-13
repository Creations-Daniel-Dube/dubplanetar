#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-12    *
#***********************************************
#!/usr/bin/env python3
"""Compile Qt .ts translation files into .qm binaries."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

LOCALES = ("fr", "en", "es", "de")
TRANSLATIONS_DIR = Path(__file__).resolve().parents[1] / "src" / "dub_planetar" / "translations"


def _find_lrelease() -> str | None:
    """Résout pyside6-lrelease via le venv courant, PATH, ou le paquet PySide6."""
    exe_name = "pyside6-lrelease.exe" if sys.platform.startswith("win") else "pyside6-lrelease"
    candidates: list[Path] = [
        Path(sys.executable).resolve().parent / exe_name,
    ]
    which = shutil.which("pyside6-lrelease")
    if which:
        candidates.append(Path(which))
    try:
        import PySide6

        pyside_dir = Path(PySide6.__file__).resolve().parent
        candidates.append(pyside_dir / "lrelease")
        candidates.append(pyside_dir / "lrelease.exe")
    except ImportError:
        pass

    for path in candidates:
        if path.is_file():
            return str(path)
    return None


def main() -> int:
    lrelease = _find_lrelease()
    if lrelease is None:
        print(
            "ERREUR : pyside6-lrelease introuvable.\n"
            "Installez PySide6 dans l'environnement actif "
            "(ex. .venv) puis réessayez.",
            file=sys.stderr,
        )
        return 1

    failed = False
    for locale in LOCALES:
        ts_path = TRANSLATIONS_DIR / f"dub_planetar_{locale}.ts"
        qm_path = TRANSLATIONS_DIR / f"dub_planetar_{locale}.qm"
        if not ts_path.is_file():
            print(f"Missing: {ts_path}", file=sys.stderr)
            failed = True
            continue
        result = subprocess.run(
            [lrelease, str(ts_path), "-qm", str(qm_path)],
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(result.stderr or result.stdout, file=sys.stderr)
            failed = True
            continue
        print(f"Compiled {qm_path.name}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
