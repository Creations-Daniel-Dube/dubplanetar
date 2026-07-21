#!/usr/bin/env python3
#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
"""Installe DubPlanetar : crée un .venv natif selon l'OS, puis les dépendances.

Le dossier .venv est local à la machine (exclu de Git) et ne doit jamais être
copié entre Windows et Linux — toujours le recréer sur l'OS cible.
"""
from __future__ import annotations

import os
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


def _banner(title: str) -> None:
    line = "=" * 56
    print(f"\n{line}")
    print(f"  {title}")
    print(f"{line}")


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


def _user_desktop() -> Path:
    """Répertoire Bureau de l'utilisateur (Desktop / Bureau / xdg)."""
    if is_windows():
        desktop = Path.home() / "Desktop"
        if desktop.is_dir():
            return desktop
        # Fallback OneDrive / chemins localisés via PowerShell
        try:
            result = subprocess.run(
                [
                    "powershell",
                    "-NoProfile",
                    "-Command",
                    "[Environment]::GetFolderPath('Desktop')",
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            path = (result.stdout or "").strip()
            if path:
                return Path(path)
        except OSError:
            pass
        return desktop

    try:
        result = subprocess.run(
            ["xdg-user-dir", "DESKTOP"],
            check=False,
            capture_output=True,
            text=True,
        )
        path = (result.stdout or "").strip()
        if path and path != Path.home().as_posix():
            return Path(path)
    except OSError:
        pass
    for name in ("Desktop", "Bureau"):
        candidate = Path.home() / name
        if candidate.is_dir():
            return candidate
    return Path.home() / "Desktop"


def _install_desktop_shortcut() -> Path | None:
    """Place un lanceur sur le Bureau (chemins absolus vers le projet)."""
    desktop = _user_desktop()
    try:
        desktop.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        print(f"AVERTISSEMENT : impossible de créer le Bureau ({desktop}) : {exc}")
        return None

    if is_windows():
        target = ROOT / "Lancer-DubPlanetar.bat"
        if not target.is_file():
            target = ROOT / "launch_dubplanetar.py"
        icon_file = ROOT / "images" / "dubPlanetar.ico"
        if not icon_file.is_file():
            icon_file = ROOT / "images" / "sun_moon.png"
        shortcut = desktop / "DubPlanetar.lnk"
        icon_line = (
            f"$s.IconLocation = '{icon_file}'; "
            if icon_file.is_file()
            else ""
        )
        ps = (
            "$ws = New-Object -ComObject WScript.Shell; "
            f"$s = $ws.CreateShortcut('{shortcut}'); "
            f"$s.TargetPath = '{target}'; "
            f"$s.WorkingDirectory = '{ROOT}'; "
            "$s.Description = 'DubPlanetar'; "
            f"{icon_line}"
            "$s.Save()"
        )
        result = subprocess.run(
            ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps],
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0 or not shortcut.is_file():
            detail = (result.stderr or result.stdout or "").strip()
            print(f"AVERTISSEMENT : raccourci Bureau non créé : {detail or 'échec PowerShell'}")
            return None
        return shortcut

    launch_sh = ROOT / "launch-dubplanetar.sh"
    if not launch_sh.is_file():
        print("AVERTISSEMENT : launch-dubplanetar.sh introuvable — pas de raccourci Bureau.")
        return None

    shortcut = desktop / "DubPlanetar.desktop"
    icon = "applications-science"
    for candidate in (
        ROOT / "images" / "sun_moon.png",
        ROOT / "images" / "dubPlanetar.ico",
        ROOT / "images" / "dubplanetar.png",
        ROOT / "images" / "icon.png",
    ):
        if candidate.is_file():
            icon = candidate.as_posix()
            break

    content = (
        "[Desktop Entry]\n"
        "Version=1.0\n"
        "Type=Application\n"
        "Name=DubPlanetar\n"
        "GenericName=SeeStar planetary stacking\n"
        "Comment=GPU Sun/Moon stacking for SeeStar RAW AVI videos\n"
        "Comment[fr]=Empilement GPU Soleil/Lune pour vidéos RAW SeeStar\n"
        f"Exec={launch_sh.as_posix()}\n"
        f"Path={ROOT.as_posix()}\n"
        f"Icon={icon}\n"
        "Terminal=false\n"
        "Categories=Science;Astronomy;\n"
        "StartupNotify=true\n"
        "Keywords=astronomy;stacking;seestar;sun;moon;\n"
        "Keywords[fr]=astronomie;empilement;seestar;soleil;lune;\n"
    )
    try:
        shortcut.write_text(content, encoding="utf-8")
        shortcut.chmod(shortcut.stat().st_mode | 0o111)
    except OSError as exc:
        print(f"AVERTISSEMENT : impossible d'écrire {shortcut} : {exc}")
        return None

    # GNOME / Ubuntu : marquer comme fiable pour le double-clic.
    if shutil.which("gio"):
        subprocess.run(
            ["gio", "set", str(shortcut), "metadata::trusted", "true"],
            check=False,
            capture_output=True,
            text=True,
        )
    return shortcut


def _launch_application(py_venv: Path) -> None:
    """Démarre DubPlanetar en arrière-plan pour valider l'installation."""
    python = venv_python(gui=True, root=ROOT)
    if not python.is_file():
        python = py_venv
    if not python.is_file():
        print("AVERTISSEMENT : Python du .venv introuvable — lancement annulé.")
        return

    argv = [str(python), "-m", "dub_planetar"]
    try:
        if is_windows():
            # DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
            flags = 0x00000008 | 0x00000200
            subprocess.Popen(
                argv,
                cwd=str(ROOT),
                creationflags=flags,
                close_fds=True,
            )
        else:
            subprocess.Popen(
                argv,
                cwd=str(ROOT),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
        print("DubPlanetar a été lancé — la fenêtre de l'application devrait s'ouvrir.")
    except OSError as exc:
        print(f"AVERTISSEMENT : impossible de lancer l'application : {exc}")
        print(f"Vous pouvez la démarrer manuellement : {launch_hint()}")


def main() -> int:
    _banner(f"DubPlanetar — installation ({os_label()})")
    print(f"Dossier : {ROOT}")
    print("Statut  : en cours…")

    _step("1/7 — Recherche de Python 3.11+")
    python_cmd, version = _find_python()
    print(f"Python {version} OK ({python_cmd})")

    py_venv = venv_python(root=ROOT)
    _step("2/7 — Création de l'environnement virtuel (.venv)")
    if not py_venv.is_file():
        _run([python_cmd, "-m", "venv", ".venv"])
        print(f"Environnement virtuel créé ({py_venv.parent.name}/).")
    else:
        print("Environnement virtuel déjà présent.")

    if not py_venv.is_file():
        print("ERREUR : impossible de trouver le Python du .venv", file=sys.stderr)
        _banner("ÉCHEC DE L'INSTALLATION")
        return 1

    _step("3/7 — Mise à jour de pip")
    _run([py_venv, "-m", "pip", "install", "--upgrade", "pip"])

    _step("4/7 — Installation des dépendances (requirements.txt)")
    result = _run([py_venv, "-m", "pip", "install", "-r", "requirements.txt"], check=False)
    if result.returncode != 0:
        print("ERREUR : échec de pip install -r requirements.txt", file=sys.stderr)
        _banner("ÉCHEC DE L'INSTALLATION")
        return 1

    _step("5/7 — Installation de DubPlanetar (mode editable)")
    result = _run([py_venv, "-m", "pip", "install", "-e", "."], check=False)
    if result.returncode != 0:
        print("ERREUR : échec de pip install -e .", file=sys.stderr)
        _banner("ÉCHEC DE L'INSTALLATION")
        return 1

    _step("6/7 — Compilation des traductions (.ts -> .qm)")
    result = _run([py_venv, "scripts/compile_translations.py"], check=False)
    if result.returncode != 0:
        print("AVERTISSEMENT : compilation des traductions échouée.")
        print("L'interface fonctionnera, mais certaines langues peuvent manquer.")

    _step("7/7 — Vérification CUDA / GPU")
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

    _step("Raccourci sur le Bureau")
    shortcut = _install_desktop_shortcut()
    if shortcut is not None:
        print(f"Lanceur créé : {shortcut}")
        if not is_windows():
            print("(Si le double-clic est bloqué : clic droit → Autoriser le lancement.)")
    else:
        print(f"Raccourci non placé — vous pouvez toujours lancer via : {launch_hint()}")

    _banner("INSTALLATION TERMINÉE AVEC SUCCÈS")
    print(f"Dossier installé : {ROOT}")
    if shortcut is not None:
        print(f"Bureau          : {shortcut.name}")
    print("\nLancement de l'application pour vérifier que tout fonctionne…")
    _launch_application(py_venv)
    print("\nNote : le dossier .venv est local et exclu de Git — ne le copiez pas entre OS.")
    return 0


if __name__ == "__main__":
    # Évite un buffer silencieux quand le script tourne dans un terminal double-clic.
    try:
        sys.stdout.reconfigure(line_buffering=True)  # type: ignore[attr-defined]
        sys.stderr.reconfigure(line_buffering=True)  # type: ignore[attr-defined]
    except Exception:
        os.environ.setdefault("PYTHONUNBUFFERED", "1")
    raise SystemExit(main())
