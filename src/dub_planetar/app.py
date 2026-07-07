#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-07    *
#***********************************************
from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
import time

from PySide6.QtCore import Qt, QSettings, QThread, QTimer
from PySide6.QtGui import QIcon, QImage, QKeySequence, QPixmap, QShortcut
from PySide6.QtWidgets import (
    QApplication,
    QFileDialog,
    QFormLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QMainWindow,
    QMessageBox,
    QPushButton,
    QProgressBar,
    QCheckBox,
    QComboBox,
    QDoubleSpinBox,
    QSpinBox,
    QVBoxLayout,
    QWidget,
)

from dub_planetar.i18n import (
    DEFAULT_LOCALE,
    PipelineError,
    get_current_locale,
    install_translator,
    next_locale,
    set_locale,
    tr_args,
    tr_pipeline,
)
from dub_planetar.pipeline.stacker import StackResult, StackSettings, check_cuda_available
from dub_planetar.worker import StackWorker

_SETTINGS_ORG = "DubPlanetar"
_SETTINGS_APP = "dubplanetar"
_LEGACY_SETTINGS_ORG = "DiscStacker"
_LEGACY_SETTINGS_APP = "disc-stacker"
_APP_ICON = Path(__file__).resolve().parent.parent.parent / "images" / "dubPlanetar.ico"

_PROFILE_DEFAULTS: dict[str, dict[str, object]] = {
    "sun": {
        "keep_ratio": 50.0,
        "max_frames": 0,
        "debayer": True,
        "bayer_pattern": "AUTO",
        "white_balance": False,
        "auto_crop": True,
        "crop_margin": 5.0,
        "drizzle": True,
        "flatten_strength": 70.0,
        "gamma": 1.2,
        "sharpen_amount": 2.0,
        "sharpen_radius": 1.0,
        "solar_stretch": 0.0,
        "subtract_background": True,
        "black_point": 0.1,
        "protect_highlights": True,
    },
    "moon": {
        "keep_ratio": 50.0,
        "max_frames": 0,
        "debayer": True,
        "bayer_pattern": "AUTO",
        "white_balance": True,
        "auto_crop": True,
        "crop_margin": 5.0,
        "drizzle": True,
        "flatten_strength": 0.0,
        "gamma": 1.2,
        "sharpen_amount": 2.0,
        "sharpen_radius": 1.0,
        "solar_stretch": 0.0,
        "subtract_background": True,
        "black_point": 0.1,
        "protect_highlights": True,
    },
}


def _migrate_legacy_settings() -> None:
    """Importe les profils Soleil/Lune sauvegardés sous Disc Stacker."""
    settings = QSettings(_SETTINGS_ORG, _SETTINGS_APP)
    if settings.value("ui/migrated_from_disc_stacker"):
        return

    legacy = QSettings(_LEGACY_SETTINGS_ORG, _LEGACY_SETTINGS_APP)
    has_profiles = legacy.value("profiles/sun/keep_ratio") is not None
    has_legacy_ui = legacy.value("ui/keep_ratio") is not None
    if not has_profiles and not has_legacy_ui:
        return

    for target in _PROFILE_DEFAULTS:
        profile_prefix = f"profiles/{target}/"
        for key in _PROFILE_DEFAULTS[target]:
            value = legacy.value(f"{profile_prefix}{key}")
            if value is not None:
                settings.setValue(f"{profile_prefix}{key}", value)

        path_prefix = f"paths/{target}/"
        for path_key in ("last_file", "last_dir"):
            value = legacy.value(f"{path_prefix}{path_key}")
            if value is not None:
                settings.setValue(f"{path_prefix}{path_key}", value)

    active = legacy.value("ui/active_target")
    if active is not None:
        settings.setValue("ui/active_target", active)

    geometry = legacy.value("ui/geometry")
    if geometry is not None:
        settings.setValue("ui/geometry", geometry)

    for path_key in ("last_file", "last_dir"):
        value = legacy.value(f"paths/{path_key}")
        if value is not None:
            settings.setValue(f"paths/sun/{path_key}", value)

    if settings.value("profiles/sun/keep_ratio") is None:
        return

    settings.setValue("ui/migrated_from_disc_stacker", True)
    settings.sync()


class MainWindow(QMainWindow):
    def __init__(self) -> None:
        super().__init__()
        if _APP_ICON.is_file():
            self.setWindowIcon(QIcon(str(_APP_ICON)))
        self.resize(1200, 720)

        self._thread: QThread | None = None
        self._worker: StackWorker | None = None
        self._restoring_settings = False
        self._switching_profile = False
        self._current_target = "sun"
        self._start_time: float | None = None
        self._elapsed_timer = QTimer(self)
        self._elapsed_timer.setInterval(100)
        self._elapsed_timer.timeout.connect(self._update_elapsed)
        self._gpu_status_key, self._gpu_status_args = check_cuda_available()
        self._form_labels: dict[str, QLabel] = {}
        self._last_stage_key: str | None = None
        self._current_locale = DEFAULT_LOCALE

        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QHBoxLayout(central)
        left_col = QVBoxLayout()
        right_col = QVBoxLayout()

        self.gpu_label = QLabel()
        self.gpu_label.setWordWrap(True)
        left_col.addWidget(self.gpu_label)

        self.target_group = QGroupBox()
        target_layout = QHBoxLayout(self.target_group)
        self._lbl_target = QLabel()
        target_layout.addWidget(self._lbl_target)
        self.target_combo = QComboBox()
        self.target_combo.addItem("", "sun")
        self.target_combo.addItem("", "moon")
        self.target_combo.currentIndexChanged.connect(self._on_target_changed)
        target_layout.addWidget(self.target_combo, stretch=1)
        left_col.addWidget(self.target_group)

        self.file_group = QGroupBox()
        file_layout = QHBoxLayout(self.file_group)
        self.input_edit = QLineEdit()
        self.browse_btn = QPushButton()
        self.browse_btn.clicked.connect(self._browse_input)
        file_layout.addWidget(self.input_edit)
        file_layout.addWidget(self.browse_btn)
        left_col.addWidget(self.file_group)

        self.settings_group = QGroupBox()
        form = QFormLayout(self.settings_group)

        self.keep_ratio = QDoubleSpinBox()
        self.keep_ratio.setRange(10, 100)
        self.keep_ratio.setSingleStep(5)
        self.keep_ratio.setSuffix(" %")
        self.keep_ratio.setDecimals(0)
        self.keep_ratio.setValue(50)
        form.addRow(self._make_form_label("keep_ratio"), self.keep_ratio)

        self.max_frames = QSpinBox()
        self.max_frames.setRange(0, 100000)
        self.max_frames.setValue(0)
        form.addRow(self._make_form_label("max_frames"), self.max_frames)

        self.debayer_check = QCheckBox()
        self.debayer_check.setChecked(True)
        form.addRow("", self.debayer_check)

        self.bayer_combo = QComboBox()
        for value in ("AUTO", "GRBG", "GBRG", "BGGR", "RGGB"):
            self.bayer_combo.addItem("", value)
        form.addRow(self._make_form_label("bayer_pattern"), self.bayer_combo)

        self.wb_check = QCheckBox()
        self.wb_check.setChecked(True)
        form.addRow("", self.wb_check)

        self.crop_check = QCheckBox()
        self.crop_check.setChecked(True)
        form.addRow("", self.crop_check)

        self.crop_margin = QDoubleSpinBox()
        self.crop_margin.setRange(1, 30)
        self.crop_margin.setSingleStep(1)
        self.crop_margin.setSuffix(" %")
        self.crop_margin.setDecimals(0)
        self.crop_margin.setValue(5)
        form.addRow(self._make_form_label("crop_margin"), self.crop_margin)

        self.drizzle_check = QCheckBox()
        self.drizzle_check.setChecked(True)
        form.addRow("", self.drizzle_check)

        left_col.addWidget(self.settings_group)

        self.tone_group = QGroupBox()
        tone_form = QFormLayout(self.tone_group)

        self.flatten_spin = QDoubleSpinBox()
        self.flatten_spin.setRange(0.0, 100.0)
        self.flatten_spin.setSingleStep(10.0)
        self.flatten_spin.setDecimals(0)
        self.flatten_spin.setSuffix(" %")
        self.flatten_spin.setValue(0.0)
        tone_form.addRow(self._make_form_label("flatten"), self.flatten_spin)

        self.gamma_spin = QDoubleSpinBox()
        self.gamma_spin.setRange(0.5, 3.0)
        self.gamma_spin.setSingleStep(0.05)
        self.gamma_spin.setDecimals(2)
        self.gamma_spin.setValue(1.20)
        tone_form.addRow(self._make_form_label("gamma"), self.gamma_spin)

        self.sharpen_spin = QDoubleSpinBox()
        self.sharpen_spin.setRange(0.0, 6.0)
        self.sharpen_spin.setSingleStep(0.25)
        self.sharpen_spin.setDecimals(2)
        self.sharpen_spin.setValue(2.0)
        tone_form.addRow(self._make_form_label("sharpen"), self.sharpen_spin)

        self.sharpen_radius_spin = QDoubleSpinBox()
        self.sharpen_radius_spin.setRange(0.5, 8.0)
        self.sharpen_radius_spin.setSingleStep(0.5)
        self.sharpen_radius_spin.setDecimals(1)
        self.sharpen_radius_spin.setValue(1.0)
        tone_form.addRow(self._make_form_label("sharpen_radius"), self.sharpen_radius_spin)

        self.solar_stretch_spin = QDoubleSpinBox()
        self.solar_stretch_spin.setRange(0.0, 50.0)
        self.solar_stretch_spin.setSingleStep(1.0)
        self.solar_stretch_spin.setDecimals(0)
        self.solar_stretch_spin.setValue(0.0)
        tone_form.addRow(self._make_form_label("solar_stretch"), self.solar_stretch_spin)

        self.bg_check = QCheckBox()
        self.bg_check.setChecked(True)
        tone_form.addRow("", self.bg_check)

        self.black_spin = QDoubleSpinBox()
        self.black_spin.setRange(0.0, 20.0)
        self.black_spin.setSingleStep(0.1)
        self.black_spin.setDecimals(1)
        self.black_spin.setSuffix(" %")
        self.black_spin.setValue(0.1)
        tone_form.addRow(self._make_form_label("black_point"), self.black_spin)

        self.protect_check = QCheckBox()
        self.protect_check.setChecked(True)
        tone_form.addRow("", self.protect_check)

        left_col.addWidget(self.tone_group)

        action_row = QHBoxLayout()
        self.stack_btn = QPushButton()
        self.stack_btn.clicked.connect(self._start_stack)
        action_row.addWidget(self.stack_btn)
        left_col.addLayout(action_row)

        self.progress = QProgressBar()
        self.progress.setRange(0, 100)
        left_col.addWidget(self.progress)

        self.preview = QLabel()
        self.preview.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.preview.setMinimumWidth(360)
        self.preview.setStyleSheet("QLabel { background: #111; color: #aaa; border: 1px solid #333; }")
        right_col.addWidget(self.preview, stretch=1)

        self.status_label = QLabel()
        right_col.addWidget(self.status_label)

        self.time_label = QLabel()
        self.time_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.time_label.setStyleSheet("QLabel { font-size: 15px; font-weight: bold; }")
        right_col.addWidget(self.time_label)

        self._left_panel = QWidget()
        self._left_panel.setLayout(left_col)
        self._right_panel = QWidget()
        self._right_panel.setLayout(right_col)

        main_layout.addWidget(self._left_panel, 0, Qt.AlignmentFlag.AlignTop)
        main_layout.addWidget(self._right_panel, 1, Qt.AlignmentFlag.AlignTop)

        self._retranslate_ui()
        QTimer.singleShot(0, self._sync_panel_heights)

        app = QApplication.instance()
        if app is not None:
            self._current_locale = get_current_locale(app)

        language_shortcut = QShortcut(QKeySequence("Ctrl+L"), self)
        language_shortcut.activated.connect(self._cycle_display_language)

        self._connect_settings_persistence()
        self._restore_ui_settings()

    def _make_form_label(self, key: str) -> QLabel:
        label = QLabel()
        self._form_labels[key] = label
        return label

    def _retranslate_ui(self) -> None:
        self.setWindowTitle(self.tr("DubPlanetar — Soleil / Lune (CUDA)"))
        self.gpu_label.setText(
            tr_args(
                self.tr("GPU : %1"),
                tr_pipeline(self._gpu_status_key, *self._gpu_status_args),
            )
        )

        self.target_group.setTitle(self.tr("Cible"))
        self._lbl_target.setText(self.tr("Objet à empiler :"))
        self._set_combo_text(self.target_combo, "sun", self.tr("☀ Soleil"))
        self._set_combo_text(self.target_combo, "moon", self.tr("☾ Lune"))

        self.file_group.setTitle(self.tr("Vidéo source"))
        self.input_edit.setPlaceholderText(self.tr("Sélectionnez un AVI RAW SeeStar…"))
        self.browse_btn.setText(self.tr("Parcourir…"))

        self.settings_group.setTitle(self.tr("Réglages"))
        self._form_labels["keep_ratio"].setText(
            self.tr("Frames conservées (moins = plus net)")
        )
        self._form_labels["max_frames"].setText(self.tr("Limite de frames (0 = toutes)"))
        self.max_frames.setSpecialValueText(self.tr("Toutes"))
        self.debayer_check.setText(self.tr("Debayer (SeeStar RAW)"))
        self._set_combo_text(self.bayer_combo, "AUTO", self.tr("Auto (recommandé)"))
        for pattern in ("GRBG", "GBRG", "BGGR", "RGGB"):
            self._set_combo_text(self.bayer_combo, pattern, pattern)
        self._form_labels["bayer_pattern"].setText(self.tr("Motif Bayer"))
        self.wb_check.setText(self.tr("Balance des blancs auto (corrige le voile vert)"))
        self.crop_check.setText(self.tr("Recadrage auto sur le disque"))
        self._form_labels["crop_margin"].setText(self.tr("Marge autour du disque"))
        self.drizzle_check.setText(self.tr("Drizzle x3 (super-résolution)"))

        self.tone_group.setTitle(
            self.tr("Tons (révèle les détails, évite les blancs brûlés)")
        )
        self._form_labels["flatten"].setText(
            self.tr("Aplatir le centre (anti-surexposition)")
        )
        self.flatten_spin.setSpecialValueText(self.tr("Désactivé"))
        self._form_labels["gamma"].setText(self.tr("Gamma (>1 révèle les détails sombres)"))
        self._form_labels["sharpen"].setText(self.tr("Netteté multi-échelle (ondelettes)"))
        self.sharpen_spin.setSpecialValueText(self.tr("Désactivé"))
        self._form_labels["sharpen_radius"].setText(self.tr("Rayon (fin=1, large=3+)"))
        self._form_labels["solar_stretch"].setText(
            self.tr("Compression Soleil (asinh, 0=off)")
        )
        self.solar_stretch_spin.setSpecialValueText(self.tr("Désactivé"))
        self.bg_check.setText(self.tr("Fond du ciel → noir (soustraction automatique)"))
        self._form_labels["black_point"].setText(self.tr("Point noir supplémentaire"))
        self.protect_check.setText(
            self.tr("Protéger les hautes lumières (ne pas brûler le centre)")
        )

        self.stack_btn.setText(self.tr("Empiler"))
        self.preview.setText(self.tr("Aperçu du résultat"))
        self._refresh_status_label()
        if self._start_time is None:
            self.time_label.setText("")

    def _refresh_status_label(self) -> None:
        if self._start_time is not None:
            if self._last_stage_key:
                self.status_label.setText(tr_pipeline(self._last_stage_key))
            else:
                self.status_label.setText(self.tr("Traitement en cours…"))
            return
        if not self.status_label.text():
            self.status_label.setText(self.tr("Prêt."))

    def _cycle_display_language(self) -> None:
        app = QApplication.instance()
        if app is None:
            return
        self._current_locale = next_locale(self._current_locale)
        set_locale(app, self._current_locale)
        self._retranslate_ui()
        if self._start_time is not None:
            self._update_elapsed()

    @staticmethod
    def _set_combo_text(combo: QComboBox, data: str, text: str) -> None:
        index = combo.findData(data)
        if index >= 0:
            combo.setItemText(index, text)

    def _sync_panel_heights(self) -> None:
        left_height = self._left_panel.sizeHint().height()
        self._right_panel.setFixedHeight(left_height)
        self.adjustSize()

    def _settings(self) -> QSettings:
        return QSettings(_SETTINGS_ORG, _SETTINGS_APP)

    @staticmethod
    def _read_bool(value: object, default: bool) -> bool:
        if value is None:
            return default
        if isinstance(value, bool):
            return value
        return str(value).lower() in {"1", "true", "yes"}

    def _connect_settings_persistence(self) -> None:
        self.keep_ratio.valueChanged.connect(self._save_ui_settings)
        self.max_frames.valueChanged.connect(self._save_ui_settings)
        self.debayer_check.stateChanged.connect(self._save_ui_settings)
        self.bayer_combo.currentIndexChanged.connect(self._save_ui_settings)
        self.wb_check.stateChanged.connect(self._save_ui_settings)
        self.crop_check.stateChanged.connect(self._save_ui_settings)
        self.crop_margin.valueChanged.connect(self._save_ui_settings)
        self.drizzle_check.stateChanged.connect(self._save_ui_settings)
        self.flatten_spin.valueChanged.connect(self._save_ui_settings)
        self.gamma_spin.valueChanged.connect(self._save_ui_settings)
        self.sharpen_spin.valueChanged.connect(self._save_ui_settings)
        self.sharpen_radius_spin.valueChanged.connect(self._save_ui_settings)
        self.solar_stretch_spin.valueChanged.connect(self._save_ui_settings)
        self.bg_check.stateChanged.connect(self._save_ui_settings)
        self.black_spin.valueChanged.connect(self._save_ui_settings)
        self.protect_check.stateChanged.connect(self._save_ui_settings)
        self.input_edit.textChanged.connect(self._save_ui_settings)

    def _active_target_key(self) -> str:
        data = self.target_combo.currentData()
        return str(data) if data else "sun"

    def _profile_prefix(self, target: str | None = None) -> str:
        key = target if target is not None else self._active_target_key()
        return f"profiles/{key}/"

    def _path_prefix(self, target: str | None = None) -> str:
        key = target if target is not None else self._active_target_key()
        return f"paths/{key}/"

    def _ui_widgets_for_block(self) -> tuple:
        return (
            self.keep_ratio,
            self.max_frames,
            self.debayer_check,
            self.bayer_combo,
            self.wb_check,
            self.crop_check,
            self.crop_margin,
            self.drizzle_check,
            self.flatten_spin,
            self.gamma_spin,
            self.sharpen_spin,
            self.sharpen_radius_spin,
            self.solar_stretch_spin,
            self.bg_check,
            self.black_spin,
            self.protect_check,
            self.input_edit,
        )

    def _collect_ui_values(self) -> dict[str, object]:
        return {
            "keep_ratio": self.keep_ratio.value(),
            "max_frames": self.max_frames.value(),
            "debayer": self.debayer_check.isChecked(),
            "bayer_pattern": self.bayer_combo.currentData(),
            "white_balance": self.wb_check.isChecked(),
            "auto_crop": self.crop_check.isChecked(),
            "crop_margin": self.crop_margin.value(),
            "drizzle": self.drizzle_check.isChecked(),
            "flatten_strength": self.flatten_spin.value(),
            "gamma": self.gamma_spin.value(),
            "sharpen_amount": self.sharpen_spin.value(),
            "sharpen_radius": self.sharpen_radius_spin.value(),
            "solar_stretch": self.solar_stretch_spin.value(),
            "subtract_background": self.bg_check.isChecked(),
            "black_point": self.black_spin.value(),
            "protect_highlights": self.protect_check.isChecked(),
        }

    def _apply_profile_values(self, values: dict[str, object]) -> None:
        self.keep_ratio.setValue(float(values["keep_ratio"]))
        self.max_frames.setValue(int(values["max_frames"]))
        self.debayer_check.setChecked(self._read_bool(values["debayer"], True))
        bayer = str(values["bayer_pattern"])
        bayer_index = self.bayer_combo.findData(bayer)
        if bayer_index >= 0:
            self.bayer_combo.setCurrentIndex(bayer_index)
        self.wb_check.setChecked(self._read_bool(values["white_balance"], True))
        self.crop_check.setChecked(self._read_bool(values["auto_crop"], True))
        self.crop_margin.setValue(float(values["crop_margin"]))
        self.drizzle_check.setChecked(self._read_bool(values["drizzle"], True))
        self.flatten_spin.setValue(float(values["flatten_strength"]))
        self.gamma_spin.setValue(float(values["gamma"]))
        self.sharpen_spin.setValue(float(values["sharpen_amount"]))
        self.sharpen_radius_spin.setValue(float(values["sharpen_radius"]))
        self.solar_stretch_spin.setValue(float(values["solar_stretch"]))
        self.bg_check.setChecked(self._read_bool(values["subtract_background"], True))
        self.black_spin.setValue(float(values["black_point"]))
        self.protect_check.setChecked(self._read_bool(values["protect_highlights"], True))

    def _profile_values_from_settings(
        self, settings: QSettings, target: str
    ) -> dict[str, object]:
        prefix = self._profile_prefix(target)
        defaults = _PROFILE_DEFAULTS[target]
        if settings.value(f"{prefix}keep_ratio") is None:
            legacy = settings.value("ui/keep_ratio")
            if legacy is not None and target == "sun":
                return self._legacy_ui_values(settings)
        return {
            key: settings.value(f"{prefix}{key}", defaults[key])
            for key in defaults
        }

    def _legacy_ui_values(self, settings: QSettings) -> dict[str, object]:
        return {
            "keep_ratio": float(settings.value("ui/keep_ratio", 50)),
            "max_frames": int(settings.value("ui/max_frames", 0)),
            "debayer": settings.value("ui/debayer", True),
            "bayer_pattern": str(settings.value("ui/bayer_pattern", "AUTO")),
            "white_balance": settings.value("ui/white_balance", True),
            "auto_crop": settings.value("ui/auto_crop", True),
            "crop_margin": float(settings.value("ui/crop_margin", 5)),
            "drizzle": settings.value("ui/drizzle", True),
            "flatten_strength": float(settings.value("ui/flatten_strength", 0.0)),
            "gamma": float(settings.value("ui/gamma", 1.2)),
            "sharpen_amount": float(settings.value("ui/sharpen_amount", 2.0)),
            "sharpen_radius": float(settings.value("ui/sharpen_radius", 1.0)),
            "solar_stretch": float(settings.value("ui/solar_stretch", 0.0)),
            "subtract_background": settings.value("ui/subtract_background", True),
            "black_point": float(settings.value("ui/black_point", 0.1)),
            "protect_highlights": settings.value("ui/protect_highlights", True),
        }

    def _save_profile(self, target: str) -> None:
        settings = self._settings()
        prefix = self._profile_prefix(target)
        for key, value in self._collect_ui_values().items():
            settings.setValue(f"{prefix}{key}", value)

        current = self.input_edit.text().strip()
        if current:
            current_path = Path(current)
            path_prefix = self._path_prefix(target)
            if current_path.is_file():
                settings.setValue(f"{path_prefix}last_file", str(current_path))
                settings.setValue(f"{path_prefix}last_dir", str(current_path.parent))
            elif current_path.parent.is_dir():
                settings.setValue(f"{path_prefix}last_dir", str(current_path.parent))

    def _load_profile(self, target: str) -> None:
        settings = self._settings()
        values = self._profile_values_from_settings(settings, target)
        self._apply_profile_values(values)

        last_file = settings.value(f"{self._path_prefix(target)}last_file", "")
        if not last_file and target == "sun":
            last_file = settings.value("paths/last_file", "")
        if last_file and Path(str(last_file)).is_file():
            self.input_edit.setText(str(last_file))
        else:
            self.input_edit.clear()

    def _on_target_changed(self) -> None:
        if self._restoring_settings or self._switching_profile:
            return

        new_target = self._active_target_key()
        if new_target == self._current_target:
            return

        self._switching_profile = True
        widgets = self._ui_widgets_for_block()
        for widget in widgets:
            widget.blockSignals(True)

        try:
            self._save_profile(self._current_target)
            self._current_target = new_target
            self._load_profile(new_target)
            self._settings().setValue("ui/active_target", new_target)
        finally:
            for widget in widgets:
                widget.blockSignals(False)
            self._switching_profile = False

    def _restore_ui_settings(self) -> None:
        _migrate_legacy_settings()
        settings = self._settings()
        self._restoring_settings = True

        widgets = self._ui_widgets_for_block()
        self.target_combo.blockSignals(True)
        for widget in widgets:
            widget.blockSignals(True)

        try:
            active = str(settings.value("ui/active_target", "sun"))
            if active not in _PROFILE_DEFAULTS:
                active = "sun"
            self._current_target = active

            target_index = self.target_combo.findData(active)
            if target_index >= 0:
                self.target_combo.setCurrentIndex(target_index)

            self._load_profile(active)

            geometry = settings.value("ui/geometry")
            if geometry is not None:
                self.restoreGeometry(geometry)
        finally:
            self.target_combo.blockSignals(False)
            for widget in widgets:
                widget.blockSignals(False)
            self._restoring_settings = False

    def _save_ui_settings(self) -> None:
        if self._restoring_settings or self._switching_profile:
            return

        settings = self._settings()
        self._save_profile(self._current_target)
        settings.setValue("ui/active_target", self._current_target)
        settings.setValue("ui/geometry", self.saveGeometry())

    def _last_directory(self) -> str:
        settings = self._settings()
        last_dir = settings.value(f"{self._path_prefix()}last_dir", "")
        if last_dir and Path(str(last_dir)).is_dir():
            return str(last_dir)
        legacy = settings.value("paths/last_dir", "")
        if legacy and Path(str(legacy)).is_dir():
            return str(legacy)
        return str(Path.home())

    def _save_last_path(self, path: Path) -> None:
        if path.is_file():
            self.input_edit.setText(str(path))
        self._save_ui_settings()

    def _browse_input(self) -> None:
        start_dir = self._last_directory()
        current = self.input_edit.text().strip()
        if current:
            current_path = Path(current)
            if current_path.parent.is_dir():
                start_dir = str(current_path.parent)

        path, _ = QFileDialog.getOpenFileName(
            self,
            self.tr("Choisir une vidéo AVI"),
            start_dir,
            self.tr("Vidéos (*.avi *.AVI);;Tous les fichiers (*.*)"),
        )
        if path:
            self._save_last_path(Path(path))

    def closeEvent(self, event) -> None:  # noqa: N802
        self._save_ui_settings()
        super().closeEvent(event)

    def _start_stack(self) -> None:
        input_path = Path(self.input_edit.text().strip())
        if not input_path.exists():
            QMessageBox.warning(
                self,
                self.tr("Fichier manquant"),
                self.tr("Sélectionnez un fichier AVI valide."),
            )
            return

        self._save_last_path(input_path)

        output_path = input_path.with_name(f"{input_path.stem}_stacked.tiff")
        settings = StackSettings(
            keep_ratio=self.keep_ratio.value() / 100.0,
            debayer=self.debayer_check.isChecked(),
            white_balance=self.wb_check.isChecked(),
            auto_crop=self.crop_check.isChecked(),
            crop_margin_ratio=self.crop_margin.value() / 100.0,
            drizzle_scale=3 if self.drizzle_check.isChecked() else 1,
            bayer_pattern=self.bayer_combo.currentData(),
            black_point=self.black_spin.value(),
            white_point=100.0 if self.protect_check.isChecked() else 99.7,
            gamma=self.gamma_spin.value(),
            subtract_background=self.bg_check.isChecked(),
            solar_stretch=self.solar_stretch_spin.value(),
            sharpen_amount=self.sharpen_spin.value(),
            sharpen_radius=self.sharpen_radius_spin.value(),
            flatten_strength=self.flatten_spin.value() / 100.0,
            max_frames=self.max_frames.value() or None,
        )

        self.stack_btn.setEnabled(False)
        self.progress.setValue(0)
        self._last_stage_key = None
        self.status_label.setText(self.tr("Traitement en cours…"))

        self._start_time = time.monotonic()
        self.time_label.setText(
            tr_args(self.tr("Temps écoulé : %1"), self._format_duration(0.0))
        )
        self._elapsed_timer.start()

        self._thread = QThread()
        self._worker = StackWorker(input_path, output_path, settings)
        self._worker.moveToThread(self._thread)

        self._thread.started.connect(self._worker.run)
        self._worker.progress.connect(self._on_progress)
        self._worker.finished.connect(self._on_finished)
        self._worker.failed.connect(self._on_failed)
        self._worker.finished.connect(self._thread.quit)
        self._worker.failed.connect(self._thread.quit)
        self._thread.finished.connect(self._cleanup_thread)

        self._thread.start()

    def _format_duration(self, seconds: float) -> str:
        if seconds >= 60:
            minutes = int(seconds // 60)
            secs = seconds - minutes * 60
            return tr_args(self.tr("%1 min %2 s"), minutes, f"{secs:04.1f}")
        return tr_args(self.tr("%1 s"), f"{seconds:.1f}")

    def _update_elapsed(self) -> None:
        if self._start_time is None:
            return
        elapsed = time.monotonic() - self._start_time
        self.time_label.setText(
            tr_args(self.tr("Temps écoulé : %1"), self._format_duration(elapsed))
        )

    def _stop_timer(self) -> float:
        self._elapsed_timer.stop()
        if self._start_time is None:
            return 0.0
        elapsed = time.monotonic() - self._start_time
        self._start_time = None
        return elapsed

    def _on_progress(self, stage: str, value: float) -> None:
        self._last_stage_key = stage
        self.progress.setValue(int(value * 100))
        self.status_label.setText(tr_pipeline(stage))

    def _on_finished(self, result: StackResult) -> None:
        elapsed = self._stop_timer()
        self._last_stage_key = None
        self.stack_btn.setEnabled(True)
        bayer_info = (
            tr_args(self.tr(", Bayer %1"), result.bayer_pattern)
            if result.bayer_pattern
            else ""
        )
        self.status_label.setText(
            tr_args(
                self.tr("Terminé — %1/%2 frames%3 → %4"),
                result.kept_frames,
                result.frame_count,
                bayer_info,
                result.output_path,
            )
        )
        self.time_label.setText(
            tr_args(self.tr("Temps total : %1"), self._format_duration(elapsed))
        )
        self._show_preview(result.output_path, rgb=result.rgb)

    def _format_error(self, error: object) -> str:
        if isinstance(error, PipelineError):
            return tr_pipeline(error.key, *error.args)
        return str(error)

    def _on_failed(self, error: object) -> None:
        elapsed = self._stop_timer()
        self._last_stage_key = None
        self.stack_btn.setEnabled(True)
        self.status_label.setText(self.tr("Erreur"))
        self.time_label.setText(
            tr_args(self.tr("Arrêté après : %1"), self._format_duration(elapsed))
        )
        QMessageBox.critical(
            self,
            self.tr("Échec de l'empilement"),
            self._format_error(error),
        )

    def _cleanup_thread(self) -> None:
        if self._worker is not None:
            self._worker.deleteLater()
            self._worker = None
        if self._thread is not None:
            self._thread.deleteLater()
            self._thread = None

    def _show_preview(self, path: Path, *, rgb: bool) -> None:
        import tifffile

        image = tifffile.imread(str(path))
        preview = self._to_qpixmap(image, rgb=rgb)
        self.preview.setPixmap(
            preview.scaled(
                self.preview.size(),
                Qt.AspectRatioMode.KeepAspectRatio,
                Qt.TransformationMode.SmoothTransformation,
            )
        )

    @staticmethod
    def _to_8bit(image: np.ndarray) -> np.ndarray:
        if image.dtype == np.uint8:
            return image
        if image.dtype == np.uint16:
            return (image.astype(np.float32) / 257.0).astype(np.uint8)
        data = image.astype(np.float32)
        peak = float(data.max())
        if peak <= 0:
            return np.zeros(image.shape, dtype=np.uint8)
        return np.clip(data / peak * 255.0, 0, 255).astype(np.uint8)

    @classmethod
    def _to_qpixmap(cls, image: np.ndarray, *, rgb: bool) -> QPixmap:
        if rgb:
            preview = np.ascontiguousarray(cls._to_8bit(image))
            h, w, _ = preview.shape
            bytes_per_line = 3 * w
            qimage = QImage(preview.data, w, h, bytes_per_line, QImage.Format.Format_RGB888)
            return QPixmap.fromImage(qimage.copy())

        mono = np.ascontiguousarray(cls._to_8bit(image))
        h, w = mono.shape
        bytes_per_line = w
        qimage = QImage(mono.data, w, h, bytes_per_line, QImage.Format.Format_Grayscale8)
        return QPixmap.fromImage(qimage.copy())


def run_app(existing_app: QApplication | None = None) -> None:
    QApplication.setOrganizationName(_SETTINGS_ORG)
    QApplication.setApplicationName(_SETTINGS_APP)
    app = existing_app or QApplication(sys.argv)
    install_translator(app)
    if _APP_ICON.is_file():
        app.setWindowIcon(QIcon(str(_APP_ICON)))
    window = MainWindow()
    window.show()
    sys.exit(app.exec())
