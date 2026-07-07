#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-05    *
#***********************************************
#!/usr/bin/env python3
"""Compile Qt .ts translation files into .qm binaries."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

LOCALES = ("fr", "en", "es", "de")
TRANSLATIONS_DIR = Path(__file__).resolve().parents[1] / "src" / "dub_planetar" / "translations"


def main() -> int:
    failed = False
    for locale in LOCALES:
        ts_path = TRANSLATIONS_DIR / f"dub_planetar_{locale}.ts"
        qm_path = TRANSLATIONS_DIR / f"dub_planetar_{locale}.qm"
        if not ts_path.is_file():
            print(f"Missing: {ts_path}", file=sys.stderr)
            failed = True
            continue
        result = subprocess.run(
            ["pyside6-lrelease", str(ts_path), "-qm", str(qm_path)],
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
