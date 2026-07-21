#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
from __future__ import annotations

import sys
from pathlib import Path


def is_windows() -> bool:
    return sys.platform.startswith("win")


def is_linux() -> bool:
    return sys.platform.startswith("linux")


def project_root() -> Path:
    return Path(__file__).resolve().parents[2]


def venv_dir(root: Path | None = None) -> Path:
    return (root or project_root()) / ".venv"


def venv_python(*, gui: bool = False, root: Path | None = None) -> Path:
    """Chemin du Python du .venv natif (Scripts/ sous Windows, bin/ sous Unix)."""
    base = venv_dir(root)
    if is_windows():
        scripts = base / "Scripts"
        if gui:
            pythonw = scripts / "pythonw.exe"
            if pythonw.is_file():
                return pythonw
        return scripts / "python.exe"
    return base / "bin" / "python"


def venv_pip(root: Path | None = None) -> Path:
    base = venv_dir(root)
    if is_windows():
        return base / "Scripts" / "pip.exe"
    return base / "bin" / "pip"


def install_script_command(root: Path | None = None) -> str:
    """Commande recommandée pour lancer l'installateur sur l'OS courant."""
    base = root or project_root()
    if is_windows():
        bat = base / "Installer-DubPlanetar.bat"
        if bat.is_file():
            return "double-clic sur Installer-DubPlanetar.bat"
        ps1 = base / "install-dubplanetar.ps1"
        if ps1.is_file():
            return ".\\install-dubplanetar.ps1"
        return "python install_dubplanetar.py"
    desktop = base / "Installer DubPlanetar.desktop"
    if desktop.is_file():
        return "double-clic sur « Installer DubPlanetar »"
    sh = base / "install-dubplanetar.sh"
    if sh.is_file():
        return "./install-dubplanetar.sh"
    return "python3 install_dubplanetar.py"


def manual_install_steps() -> str:
    if is_windows():
        return (
            "Depuis le dossier du projet :\n"
            "  python -m venv .venv\n"
            "  .\\.venv\\Scripts\\pip install -r requirements.txt\n"
            "  .\\.venv\\Scripts\\pip install -e ."
        )
    return (
        "Depuis le dossier du projet :\n"
        "  python3 -m venv .venv\n"
        "  .venv/bin/pip install -r requirements.txt\n"
        "  .venv/bin/pip install -e ."
    )


def install_hint(root: Path | None = None) -> str:
    base = root or project_root()
    command = install_script_command(base)
    if is_windows():
        preferred = base / "Installer-DubPlanetar.bat"
        if not preferred.is_file():
            preferred = base / "install-dubplanetar.ps1"
        if not preferred.is_file():
            preferred = base / "install_dubplanetar.py"
    else:
        preferred = base / "Installer DubPlanetar.desktop"
        if not preferred.is_file():
            preferred = base / "install-dubplanetar.sh"
        if not preferred.is_file():
            preferred = base / "install_dubplanetar.py"
    if preferred.is_file():
        return (
            f"Lancez l'installateur :\n"
            f"  {command}\n\n"
            f"Chemin : {preferred}"
        )
    return manual_install_steps()


def compile_translations_hint() -> str:
    return "python scripts/compile_translations.py"


def launch_hint() -> str:
    if is_windows():
        if (project_root() / "Lancer-DubPlanetar.bat").is_file():
            return "double-clic sur Lancer-DubPlanetar.bat"
        return ".\\launch-dubplanetar.ps1"
    if (project_root() / "DubPlanetar.desktop").is_file():
        return "double-clic sur DubPlanetar.desktop"
    return "./launch-dubplanetar.sh"


def os_label() -> str:
    if is_windows():
        return "Windows"
    if is_linux():
        return "Linux"
    return sys.platform
