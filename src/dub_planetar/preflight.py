#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-07    *
#***********************************************
from __future__ import annotations

import importlib.util
import sys
from dataclasses import dataclass
from pathlib import Path

_MIN_PYTHON = (3, 11)
_PROJECT_ROOT = Path(__file__).resolve().parents[2]
_INSTALL_SCRIPT = _PROJECT_ROOT / "install-dubplanetar.ps1"

_DEPENDENCIES: tuple[tuple[str, str, str], ...] = (
    ("PySide6", "PySide6", "Interface graphique Qt"),
    ("numpy", "numpy", "Calcul numérique"),
    ("cv2", "opencv-python-headless", "Lecture vidéo AVI"),
    ("tifffile", "tifffile", "Export TIFF 16 bits"),
    ("cupy", "cupy-cuda12x", "Calcul GPU CUDA"),
)


@dataclass(frozen=True)
class PreflightIssue:
    level: str  # "error" | "warning"
    title: str
    detail: str


def _python_issue() -> PreflightIssue | None:
    if sys.version_info >= _MIN_PYTHON:
        return None
    need = ".".join(str(part) for part in _MIN_PYTHON)
    have = ".".join(str(part) for part in sys.version_info[:3])
    return PreflightIssue(
        "error",
        "Version Python insuffisante",
        f"Python {need}+ requis, version actuelle : {have}.",
    )


def _dependency_issues() -> list[PreflightIssue]:
    issues: list[PreflightIssue] = []
    for module_name, pip_name, label in _DEPENDENCIES:
        if importlib.util.find_spec(module_name) is not None:
            continue
        issues.append(
            PreflightIssue(
                "error",
                f"Composant manquant : {label}",
                f"Le module « {module_name} » n'est pas installé.\n"
                f"Paquet pip : {pip_name}",
            )
        )
    return issues


def _cuda_issue() -> PreflightIssue | None:
    if importlib.util.find_spec("cupy") is None:
        return None
    try:
        import cupy as cp

        device = cp.cuda.Device(0)
        device.use()
        cp.cuda.runtime.getDeviceProperties(0)
    except Exception as exc:  # noqa: BLE001
        return PreflightIssue(
            "warning",
            "CUDA / GPU indisponible",
            f"L'empilement GPU ne fonctionnera pas.\n{exc}",
        )
    return None


def _translation_issues() -> list[PreflightIssue]:
    translations_dir = Path(__file__).resolve().parent / "translations"
    missing = [
        path.name
        for path in (
            translations_dir / "dub_planetar_fr.qm",
            translations_dir / "dub_planetar_en.qm",
            translations_dir / "dub_planetar_es.qm",
            translations_dir / "dub_planetar_de.qm",
        )
        if not path.is_file()
    ]
    if not missing:
        return []
    return [
        PreflightIssue(
            "warning",
            "Traductions incomplètes",
            "Fichiers .qm absents : "
            + ", ".join(missing)
            + "\n\nExécutez : python scripts\\compile_translations.py",
        )
    ]


def collect_preflight_issues() -> list[PreflightIssue]:
    issues: list[PreflightIssue] = []
    python_issue = _python_issue()
    if python_issue is not None:
        issues.append(python_issue)
    issues.extend(_dependency_issues())
    cuda_issue = _cuda_issue()
    if cuda_issue is not None:
        issues.append(cuda_issue)
    issues.extend(_translation_issues())
    return issues


def _install_hint() -> str:
    script = _INSTALL_SCRIPT
    if script.is_file():
        return (
            f"Exécutez le script d'installation :\n"
            f"  .\\install-dubplanetar.ps1\n\n"
            f"Chemin : {script}"
        )
    return (
        "Depuis le dossier du projet :\n"
        "  python -m venv .venv\n"
        "  .\\.venv\\Scripts\\pip install -r requirements.txt\n"
        "  .\\.venv\\Scripts\\pip install -e ."
    )


def format_preflight_message(issues: list[PreflightIssue]) -> str:
    blocks: list[str] = []
    for issue in issues:
        prefix = "ERREUR" if issue.level == "error" else "AVERTISSEMENT"
        blocks.append(f"[{prefix}] {issue.title}\n{issue.detail}")
    if any(issue.level == "error" for issue in issues):
        blocks.append(_install_hint())
    return "\n\n".join(blocks)


def has_blocking_issues(issues: list[PreflightIssue]) -> bool:
    return any(issue.level == "error" for issue in issues)


def show_preflight_dialog(app) -> bool:
    """Affiche les problèmes détectés. Retourne False si le démarrage doit être annulé."""
    from PySide6.QtWidgets import QMessageBox

    issues = collect_preflight_issues()
    if not issues:
        return True

    message = format_preflight_message(issues)
    blocking = has_blocking_issues(issues)

    if blocking:
        QMessageBox.critical(
            None,
            "DubPlanetar — composants manquants",
            message,
        )
        return False

    QMessageBox.warning(
        None,
        "DubPlanetar — avertissements au démarrage",
        message,
    )
    return True
