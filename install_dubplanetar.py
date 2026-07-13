#!/usr/bin/env python3
#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-12    *
#***********************************************
"""Installe DubPlanetar : crée un .venv natif selon l'OS, puis les dépendances.

Le dossier .venv est local à la machine (exclu de Git) et ne doit jamais être
copié entre Windows et Linux — toujours le recréer sur l'OS cible.
"""
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "src"))

from dub_planetar.platform_env import (  # noqa: E402
    is_windows,
    launch_hint,
    os_label,
    venv_python,
)

_MIN_PYTHON = (3, 11)


def _step(message: str) -> None:
    print(f"\n==> {message}")


def _run(cmd: list[str | Path], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(part) for part in cmd],
        cwd=ROOT,
        check=check,
        text=True,
    )


def _find_python() -> tuple[str, str]:
    candidates = ("python", "py") if is_windows() else ("python3", "python")
    for name in candidates:
        if shutil.which(name) is None:
            continue
        try:
            result = subprocess.run(
                [name, "-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"],
                check=False,
                capture_output=True,
                text=True,
            )
        except OSError:
            continue
        if result.returncode != 0 or not result.stdout.strip():
            continue
        version = result.stdout.strip()
        parts = version.split(".")
        major, minor = int(parts[0]), int(parts[1])
        if (major, minor) >= _MIN_PYTHON[:2]:
            return name, version
        print(f"Avertissement : Python {version} trouvé, mais 3.11+ est requis.")
    raise SystemExit(
        "ERREUR : Python 3.11+ introuvable.\n"
        + (
            "Installez Python depuis https://www.python.org/downloads/\n"
            "Cochez « Add python.exe to PATH » pendant l'installation."
            if is_windows()
            else "Sous Ubuntu : sudo apt install python3 python3-venv python3-pip"
        )
    )


def main() -> int:
    print(f"DubPlanetar — installation ({os_label()})")
    print(f"Dossier : {ROOT}")

    _step("Recherche de Python 3.11+")
    python_cmd, version = _find_python()
    print(f"Python {version} OK ({python_cmd})")

    py_venv = venv_python(root=ROOT)
    _step("Création de l'environnement virtuel (.venv)")
    if not py_venv.is_file():
        _run([python_cmd, "-m", "venv", ".venv"])
        print(f"Environnement virtuel créé ({py_venv.parent.name}/).")
    else:
        print("Environnement virtuel déjà présent.")

    if not py_venv.is_file():
        print("ERREUR : impossible de trouver le Python du .venv", file=sys.stderr)
        return 1

    _step("Mise à jour de pip")
    _run([py_venv, "-m", "pip", "install", "--upgrade", "pip"])

    _step("Installation des dépendances (requirements.txt)")
    result = _run([py_venv, "-m", "pip", "install", "-r", "requirements.txt"], check=False)
    if result.returncode != 0:
        print("ERREUR : échec de pip install -r requirements.txt", file=sys.stderr)
        return 1

    _step("Installation de DubPlanetar (mode editable)")
    result = _run([py_venv, "-m", "pip", "install", "-e", "."], check=False)
    if result.returncode != 0:
        print("ERREUR : échec de pip install -e .", file=sys.stderr)
        return 1

    _step("Compilation des traductions (.ts -> .qm)")
    result = _run([py_venv, "scripts/compile_translations.py"], check=False)
    if result.returncode != 0:
        print("AVERTISSEMENT : compilation des traductions échouée.")
        print("L'interface fonctionnera, mais certaines langues peuvent manquer.")

    _step("Vérification CUDA / GPU")
    cuda_check = subprocess.run(
        [
            str(py_venv),
            "-c",
            (
                "try:\n"
                " import cupy as cp\n"
                " dev = cp.cuda.Device(0)\n"
                " dev.use()\n"
                " props = cp.cuda.runtime.getDeviceProperties(0)\n"
                " name = props['name'].decode() if isinstance(props['name'], bytes) else str(props['name'])\n"
                " mem = dev.mem_info[1] / (1024 ** 3)\n"
                " print(f'OK|{name}|{mem:.1f}')\n"
                "except Exception as exc:\n"
                " print(f'FAIL|{exc}')\n"
            ),
        ],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    output = (cuda_check.stdout or "").strip()
    if output.startswith("OK|"):
        _, name, mem = output.split("|", 2)
        print(f"GPU détecté : {name} ({mem} Go VRAM libre)")
    else:
        detail = output.removeprefix("FAIL|").strip() or (cuda_check.stderr or "").strip()
        print(f"CUDA / GPU non disponible : {detail}")
        print("DubPlanetar s'ouvrira, mais l'empilement GPU ne fonctionnera pas.")
        print("Vérifiez vos pilotes NVIDIA et CUDA 12.x.")

    print("\nInstallation terminée.")
    print("\nLancer l'application :")
    print(f"  {launch_hint()}")
    print(f"  ou : {py_venv} -m dub_planetar")
    print("\nNote : le dossier .venv est local et exclu de Git — ne le copiez pas entre OS.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
