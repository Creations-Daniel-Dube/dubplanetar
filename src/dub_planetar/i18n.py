#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-05    *
#***********************************************
from __future__ import annotations

from importlib import resources
from pathlib import Path

from PySide6.QtCore import QCoreApplication, QLocale, QTranslator
from PySide6.QtWidgets import QApplication

SUPPORTED_LOCALES = ("fr", "en", "es", "de")
DEFAULT_LOCALE = "en"
PIPELINE_CONTEXT = "Pipeline"


class PipelineError(Exception):
    def __init__(self, key: str, *args: object) -> None:
        self.key = key
        self.args = args
        super().__init__(key)


def _language_candidates() -> list[str]:
    candidates: list[str] = []
    for ui_lang in QLocale.system().uiLanguages():
        token = ui_lang.split("-")[0].split("_")[0].lower()
        if token and token not in candidates:
            candidates.append(token)
    return candidates


def _translations_dir() -> Path:
    return Path(resources.files("dub_planetar")) / "translations"


def install_translator(app: QApplication) -> str:
    translations_dir = _translations_dir()
    translator = QTranslator(app)

    for language in _language_candidates():
        if language not in SUPPORTED_LOCALES:
            continue
        qm_name = f"dub_planetar_{language}.qm"
        if translator.load(qm_name, str(translations_dir)):
            app.installTranslator(translator)
            return language

    fallback = DEFAULT_LOCALE
    if translator.load(f"dub_planetar_{fallback}.qm", str(translations_dir)):
        app.installTranslator(translator)
    return fallback


def tr_args(text: str, *args: object) -> str:
    result = text
    for index, value in enumerate(args, start=1):
        result = result.replace(f"%{index}", str(value))
    return result


def tr_pipeline(key: str, *args: object) -> str:
    message = QCoreApplication.translate(PIPELINE_CONTEXT, key)
    if not message or message == key:
        message = key
    for index, value in enumerate(args, start=1):
        message = message.replace(f"%{index}", str(value))
    return message
