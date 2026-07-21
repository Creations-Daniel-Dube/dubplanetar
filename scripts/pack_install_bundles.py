#!/usr/bin/env python3
#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
"""Génère les packs d'installation Windows et Linux sous install/.

Pour chaque OS :
  1. assemble le projet dans install/<OS>/files/
       - DubPlanetar-<ver>.zip | .tar.gz  (contenu applicatif)
       - Install.bat | Install.sh
  2. emballe ces deux fichiers dans un pack unique à télécharger :
       - install/Windows/dubPlanetar-<ver>_install.zip
       - install/Linux/dubPlanetar-<ver>_install.tar.gz

Le dossier files/ est local (hors Git) ; seuls le pack _install et les
README/LISEZMOI sont destinés à GitHub.

Usage (depuis la racine du dépôt) :
  python3 scripts/pack_install_bundles.py
"""
from __future__ import annotations

import re
import shutil
import stat
import tarfile
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INSTALL_DIR = ROOT / "install"
ARCHIVE_ROOT_NAME = "DubPlanetar"
FILES_DIR_NAME = "files"

# Fichiers / dossiers à copier à la racine de l'archive (chemins relatifs à ROOT).
INCLUDE_TOP = (
    "src",
    "scripts",
    "images",
    "requirements.txt",
    "pyproject.toml",
    "README.md",
    "README.fr.md",
    "install_dubplanetar.py",
    "install-dubplanetar.sh",
    "install-dubplanetar.ps1",
    "Installer-DubPlanetar.bat",
    "Installer DubPlanetar.desktop",
    "launch_dubplanetar.py",
    "launch-dubplanetar.sh",
    "launch-dubplanetar.ps1",
    "Lancer-DubPlanetar.bat",
    "Lancer-DubPlanetar.vbs",
    "DubPlanetar.desktop",
)

EXCLUDE_DIR_NAMES = {
    ".git",
    ".venv",
    "venv",
    "install",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "dist",
    "build",
    "output",
    ".egg-info",
}

EXCLUDE_SUFFIXES = (
    ".pyc",
    ".pyo",
    ".tiff",
    ".tif",
    ".avi",
    ".spec",
)

# Outils de packaging (inutiles sur la machine cible).
EXCLUDE_FILE_NAMES = {
    "pack_install_bundles.py",
}


def _read_version() -> str:
    text = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if not match:
        raise SystemExit("ERREUR : version introuvable dans pyproject.toml")
    return match.group(1)


def _should_skip(path: Path) -> bool:
    if path.name in EXCLUDE_DIR_NAMES:
        return True
    if path.name in EXCLUDE_FILE_NAMES:
        return True
    if path.name.endswith(".egg-info"):
        return True
    if path.suffix.lower() in EXCLUDE_SUFFIXES:
        return True
    return False


def _copy_filtered(src: Path, dst: Path) -> None:
    if src.is_file():
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)
        return

    dst.mkdir(parents=True, exist_ok=True)
    for child in src.iterdir():
        if _should_skip(child):
            continue
        target = dst / child.name
        if child.is_dir():
            _copy_filtered(child, target)
        else:
            shutil.copy2(child, target)


def _build_payload(staging: Path) -> Path:
    payload = staging / ARCHIVE_ROOT_NAME
    payload.mkdir(parents=True, exist_ok=True)
    missing: list[str] = []
    for name in INCLUDE_TOP:
        src = ROOT / name
        if not src.exists():
            missing.append(name)
            continue
        _copy_filtered(src, payload / name)
    if missing:
        raise SystemExit(
            "ERREUR : fichiers/dossiers manquants pour le pack :\n  - "
            + "\n  - ".join(missing)
        )
    return payload


def _write_payload_zip(payload: Path, zip_path: Path) -> None:
    zip_path.parent.mkdir(parents=True, exist_ok=True)
    if zip_path.exists():
        zip_path.unlink()
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for file_path in sorted(payload.rglob("*")):
            if file_path.is_file():
                arcname = file_path.relative_to(payload.parent).as_posix()
                zf.write(file_path, arcname)


def _write_payload_tar_gz(payload: Path, tar_path: Path) -> None:
    tar_path.parent.mkdir(parents=True, exist_ok=True)
    if tar_path.exists():
        tar_path.unlink()
    with tarfile.open(tar_path, "w:gz") as tf:
        tf.add(payload, arcname=ARCHIVE_ROOT_NAME)


def _write_flat_zip(files: list[Path], zip_path: Path) -> None:
    """Archive plate : Install.* + archive applicative à la racine du zip."""
    zip_path.parent.mkdir(parents=True, exist_ok=True)
    if zip_path.exists():
        zip_path.unlink()
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for file_path in files:
            zf.write(file_path, file_path.name)


def _write_flat_tar_gz(files: list[Path], tar_path: Path) -> None:
    """Archive plate : Install.* + archive applicative à la racine du tar.gz."""
    tar_path.parent.mkdir(parents=True, exist_ok=True)
    if tar_path.exists():
        tar_path.unlink()
    with tarfile.open(tar_path, "w:gz") as tf:
        for file_path in files:
            tf.add(file_path, arcname=file_path.name)


def _clean_os_dir(os_dir: Path, *, keep_names: set[str]) -> None:
    """Supprime les anciens artefacts à la racine de l'OS (hors README / packs)."""
    if not os_dir.is_dir():
        return
    for child in os_dir.iterdir():
        if child.name in keep_names:
            continue
        if child.name == FILES_DIR_NAME:
            continue
        if child.is_file() and (
            child.name.startswith("DubPlanetar-")
            or child.name.startswith("dubPlanetar-")
            or child.name in {"Install.bat", "Install.sh"}
        ):
            child.unlink()


def _reset_files_dir(files_dir: Path) -> None:
    if files_dir.exists():
        shutil.rmtree(files_dir)
    files_dir.mkdir(parents=True, exist_ok=True)


def _install_bat(archive_name: str) -> str:
    # Expand-Archive via PowerShell ; fallback tar (Windows 10+).
    return f"""@echo off
REM DubPlanetar — décompresse le pack puis lance l'installation (Windows).
setlocal EnableExtensions
cd /d "%~dp0"

set "ARCHIVE={archive_name}"
set "DEST={ARCHIVE_ROOT_NAME}"

echo ========================================================
echo   DubPlanetar — installation (Windows)
echo ========================================================
echo.

if not exist "%ARCHIVE%" (
  echo ERREUR : archive introuvable : %ARCHIVE%
  echo Placez Install.bat et %ARCHIVE% dans le meme dossier.
  echo.
  pause
  exit /b 1
)

if exist "%DEST%\\" (
  echo [1/3] Le dossier %DEST% existe deja — suppression...
  rmdir /s /q "%DEST%"
  if exist "%DEST%\\" (
    echo ERREUR : impossible de supprimer %DEST%
    pause
    exit /b 1
  )
) else (
  echo [1/3] Pret a extraire.
)

echo [2/3] Extraction de %ARCHIVE% ...
where tar >nul 2>&1
if %ERRORLEVEL%==0 (
  tar -xf "%ARCHIVE%"
) else (
  powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -LiteralPath '%ARCHIVE%' -DestinationPath '.' -Force"
)
if errorlevel 1 (
  echo ERREUR : echec de la decompression.
  pause
  exit /b 1
)
echo       Extraction terminee.

if not exist "%DEST%\\Installer-DubPlanetar.bat" (
  echo ERREUR : %DEST%\\Installer-DubPlanetar.bat introuvable apres extraction.
  pause
  exit /b 1
)

echo [3/3] Installation des dependances (venv, pip, raccourci Bureau)...
echo       Suivez la progression ci-dessous...
echo.

set "PY="
where python >nul 2>&1 && set "PY=python"
if not defined PY (
  where py >nul 2>&1 && set "PY=py -3"
)
if not defined PY (
  echo ERREUR : Python 3.11+ introuvable.
  echo Installez Python depuis https://www.python.org/downloads/
  echo Cochez « Add python.exe to PATH » pendant l'installation.
  echo.
  pause
  exit /b 1
)

pushd "%DEST%"
%PY% -u install_dubplanetar.py
set "ERR=%ERRORLEVEL%"
popd

echo.
if not "%ERR%"=="0" (
  echo ========================================================
  echo   ECHEC DE L'INSTALLATION (code %ERR%)
  echo ========================================================
) else (
  echo ========================================================
  echo   INSTALLATION TERMINEE
  echo   - Raccourci : Bureau \\ DubPlanetar
  echo   - L'application devrait s'etre ouverte
  echo ========================================================
)
echo.
pause
exit /b %ERR%
"""


def _install_sh(archive_name: str) -> str:
    return f"""#!/usr/bin/env bash
# DubPlanetar — décompresse le pack puis lance l'installation (Linux).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Double-clic sans terminal : rouvrir dans un terminal visible.
if [[ ! -t 1 ]]; then
  title="DubPlanetar — installation"
  cmd="bash \\"$SCRIPT_DIR/$(basename "$0")\\""
  for term in gnome-terminal konsole xfce4-terminal mate-terminal x-terminal-emulator xterm; do
    if command -v "$term" >/dev/null 2>&1; then
      case "$term" in
        gnome-terminal) exec gnome-terminal --title="$title" -- bash -lc "$cmd" ;;
        konsole)        exec konsole --title "$title" -e bash -lc "$cmd" ;;
        *)              exec "$term" -e bash -lc "$cmd" ;;
      esac
    fi
  done
fi

ARCHIVE="{archive_name}"
DEST="{ARCHIVE_ROOT_NAME}"

echo "========================================================"
echo "  DubPlanetar — installation (Linux)"
echo "========================================================"
echo

if [[ ! -f "$ARCHIVE" ]]; then
  echo "ERREUR : archive introuvable : $ARCHIVE"
  echo "Placez Install.sh et $ARCHIVE dans le même dossier."
  echo
  read -r -p "Appuyez sur Entrée pour fermer… " || true
  exit 1
fi

if [[ -e "$DEST" ]]; then
  echo "[1/3] Le dossier $DEST existe déjà — suppression..."
  rm -rf "$DEST"
else
  echo "[1/3] Prêt à extraire."
fi

echo "[2/3] Extraction de $ARCHIVE ..."
tar -xzf "$ARCHIVE"
echo "      Extraction terminée."

if [[ ! -f "$DEST/install-dubplanetar.sh" ]]; then
  echo "ERREUR : $DEST/install-dubplanetar.sh introuvable après extraction."
  read -r -p "Appuyez sur Entrée pour fermer… " || true
  exit 1
fi

chmod +x "$DEST/install-dubplanetar.sh" "$DEST/launch-dubplanetar.sh" \\
  "$DEST/scripts/run_desktop_action.sh" 2>/dev/null || true

echo "[3/3] Installation des dépendances (venv, pip, raccourci Bureau)..."
echo "      Suivez la progression ci-dessous…"
echo

set +e
"$DEST/install-dubplanetar.sh"
status=$?
set -e

echo
if [[ "$status" -ne 0 ]]; then
  echo "========================================================"
  echo "  ÉCHEC DE L'INSTALLATION (code $status)"
  echo "========================================================"
else
  echo "========================================================"
  echo "  INSTALLATION TERMINÉE"
  echo "  - Raccourci : Bureau / DubPlanetar"
  echo "  - L'application devrait s'être ouverte"
  echo "========================================================"
fi
echo
read -r -p "Appuyez sur Entrée pour fermer… " || true
exit "$status"
"""


def main() -> int:
    version = _read_version()
    archive_base = f"DubPlanetar-{version}"
    zip_name = f"{archive_base}.zip"
    tar_name = f"{archive_base}.tar.gz"
    win_bundle_name = f"dubPlanetar-{version}_install.zip"
    lin_bundle_name = f"dubPlanetar-{version}_install.tar.gz"

    win_dir = INSTALL_DIR / "Windows"
    lin_dir = INSTALL_DIR / "Linux"
    win_files = win_dir / FILES_DIR_NAME
    lin_files = lin_dir / FILES_DIR_NAME

    win_dir.mkdir(parents=True, exist_ok=True)
    lin_dir.mkdir(parents=True, exist_ok=True)
    _clean_os_dir(win_dir, keep_names={"README.txt", "LISEZMOI.txt", win_bundle_name})
    _clean_os_dir(lin_dir, keep_names={"README.txt", "LISEZMOI.txt", lin_bundle_name})
    _reset_files_dir(win_files)
    _reset_files_dir(lin_files)

    print(f"DubPlanetar — packaging v{version}")
    print(f"Racine : {ROOT}")

    with tempfile.TemporaryDirectory(prefix="dubplanetar-pack-") as tmp:
        staging = Path(tmp)
        print("\n==> Assemblage du contenu applicatif")
        payload = _build_payload(staging)

        zip_path = win_files / zip_name
        print(f"\n==> Archive Windows (files/) : {zip_path.relative_to(ROOT)}")
        _write_payload_zip(payload, zip_path)

        tar_path = lin_files / tar_name
        print(f"==> Archive Linux   (files/) : {tar_path.relative_to(ROOT)}")
        _write_payload_tar_gz(payload, tar_path)

    bat_path = win_files / "Install.bat"
    sh_path = lin_files / "Install.sh"
    print(f"\n==> Wrapper Windows (files/) : {bat_path.relative_to(ROOT)}")
    bat_path.write_text(_install_bat(zip_name), encoding="utf-8", newline="\r\n")
    print(f"==> Wrapper Linux   (files/) : {sh_path.relative_to(ROOT)}")
    sh_path.write_text(_install_sh(tar_name), encoding="utf-8", newline="\n")
    sh_path.chmod(sh_path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    win_bundle = win_dir / win_bundle_name
    lin_bundle = lin_dir / lin_bundle_name
    print(f"\n==> Pack unique Windows : {win_bundle.relative_to(ROOT)}")
    _write_flat_zip([bat_path, zip_path], win_bundle)
    print(f"==> Pack unique Linux   : {lin_bundle.relative_to(ROOT)}")
    _write_flat_tar_gz([sh_path, tar_path], lin_bundle)

    print("\nPacks prêts.")
    print(f"  Windows : {win_bundle}")
    print(f"  Linux   : {lin_bundle}")
    print(f"  (intermédiaire local : {win_files.name}/ et {lin_files.name}/ — hors Git)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
