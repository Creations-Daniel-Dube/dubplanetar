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


def _load_translator(language: str) -> QTranslator | None:
    translations_dir = _translations_dir()
    translator = QTranslator()
    qm_name = f"dub_planetar_{language}.qm"
    if translator.load(qm_name, str(translations_dir)):
        return translator
    return None


def set_locale(app: QApplication, language: str) -> str:
    if language not in SUPPORTED_LOCALES:
        language = DEFAULT_LOCALE

    existing = getattr(app, "_dub_planetar_translator", None)
    if existing is not None:
        app.removeTranslator(existing)

    translator = _load_translator(language)
    if translator is None:
        language = DEFAULT_LOCALE
        translator = _load_translator(language)
    if translator is not None:
        app.installTranslator(translator)
        app._dub_planetar_translator = translator

    app._dub_planetar_locale = language
    return language


def get_current_locale(app: QApplication) -> str:
    return getattr(app, "_dub_planetar_locale", DEFAULT_LOCALE)


def next_locale(current: str) -> str:
    if current not in SUPPORTED_LOCALES:
        return SUPPORTED_LOCALES[0]
    index = SUPPORTED_LOCALES.index(current)
    return SUPPORTED_LOCALES[(index + 1) % len(SUPPORTED_LOCALES)]


def install_translator(app: QApplication) -> str:
    for language in _language_candidates():
        if language in SUPPORTED_LOCALES:
            return set_locale(app, language)
    return set_locale(app, DEFAULT_LOCALE)


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
