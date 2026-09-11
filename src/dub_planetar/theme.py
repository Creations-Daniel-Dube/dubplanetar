#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Version  ----------------->   00.08.250     *
#* Dernières Modifications -->   2026-09-11    *
#***********************************************
from __future__ import annotations

from PySide6.QtGui import QColor, QPalette
from PySide6.QtWidgets import QApplication

# Arrière-plan de la fenêtre : rouge très foncé
COLOR_WINDOW = "#160505"
# Frames (Cible, Vidéo source, …) : un cran plus clair
COLOR_FRAME = "#3d1210"
# TextBox, listes, boutons, spinners : encore un cran plus clair
COLOR_CONTROL = "#622018"
COLOR_CONTROL_HOVER = "#7a2820"
COLOR_CONTROL_PRESSED = "#4a1414"
COLOR_BORDER = "#8e3224"
# Labels et titres de groupes
COLOR_LABEL = "#ffe14d"
# Texte saisi / affiché dans les contrôles
COLOR_CONTROL_TEXT = "#ff9320"
COLOR_HIGHLIGHT = "#c43a18"
COLOR_DISABLED = "#8a5a38"
COLOR_PREVIEW = "#0e0303"

_STYLESHEET = f"""
QMainWindow, QDialog, QMessageBox {{
    background-color: {COLOR_WINDOW};
    color: {COLOR_LABEL};
}}
QWidget#centralPanel, QWidget#leftPanel, QWidget#rightPanel {{
    background-color: {COLOR_WINDOW};
    color: {COLOR_LABEL};
}}
QLabel {{
    color: {COLOR_LABEL};
    background-color: transparent;
}}
QLabel:disabled {{
    color: {COLOR_DISABLED};
}}
QGroupBox {{
    background-color: {COLOR_FRAME};
    color: {COLOR_LABEL};
    border: 1px solid {COLOR_BORDER};
    border-radius: 6px;
    margin-top: 12px;
    padding: 10px 8px 8px 8px;
}}
QGroupBox::title {{
    subcontrol-origin: margin;
    subcontrol-position: top left;
    left: 10px;
    padding: 0 6px;
    color: {COLOR_LABEL};
    background-color: {COLOR_FRAME};
}}
QLineEdit, QAbstractSpinBox, QComboBox, QTextEdit, QPlainTextEdit,
QListView, QListWidget, QTreeView, QTableView, QAbstractItemView {{
    background-color: {COLOR_CONTROL};
    color: {COLOR_CONTROL_TEXT};
    border: 1px solid {COLOR_BORDER};
    border-radius: 3px;
    padding: 3px 6px;
    selection-background-color: {COLOR_HIGHLIGHT};
    selection-color: {COLOR_LABEL};
}}
QLineEdit:disabled, QAbstractSpinBox:disabled, QComboBox:disabled {{
    color: {COLOR_DISABLED};
    background-color: {COLOR_FRAME};
}}
QComboBox QAbstractItemView {{
    background-color: {COLOR_CONTROL};
    color: {COLOR_CONTROL_TEXT};
    border: 1px solid {COLOR_BORDER};
    selection-background-color: {COLOR_HIGHLIGHT};
    selection-color: {COLOR_LABEL};
}}
QPushButton {{
    background-color: {COLOR_CONTROL};
    color: {COLOR_CONTROL_TEXT};
    border: 1px solid {COLOR_BORDER};
    border-radius: 4px;
    padding: 5px 14px;
    min-height: 22px;
}}
QPushButton:hover {{
    background-color: {COLOR_CONTROL_HOVER};
}}
QPushButton:pressed {{
    background-color: {COLOR_CONTROL_PRESSED};
}}
QPushButton:disabled {{
    color: {COLOR_DISABLED};
    background-color: {COLOR_FRAME};
}}
QCheckBox, QRadioButton {{
    color: {COLOR_LABEL};
    background-color: transparent;
    spacing: 6px;
}}
QCheckBox:disabled, QRadioButton:disabled {{
    color: {COLOR_DISABLED};
}}
QProgressBar {{
    background-color: {COLOR_CONTROL};
    color: {COLOR_CONTROL_TEXT};
    border: 1px solid {COLOR_BORDER};
    border-radius: 3px;
    text-align: center;
    min-height: 16px;
}}
QProgressBar::chunk {{
    background-color: {COLOR_HIGHLIGHT};
    border-radius: 2px;
}}
QToolTip {{
    background-color: {COLOR_FRAME};
    color: {COLOR_LABEL};
    border: 1px solid {COLOR_BORDER};
    padding: 4px;
}}
QMenu {{
    background-color: {COLOR_FRAME};
    color: {COLOR_LABEL};
    border: 1px solid {COLOR_BORDER};
}}
QMenu::item:selected {{
    background-color: {COLOR_HIGHLIGHT};
    color: {COLOR_LABEL};
}}
QScrollBar:vertical, QScrollBar:horizontal {{
    background: {COLOR_FRAME};
    border: none;
}}
QScrollBar::handle:vertical, QScrollBar::handle:horizontal {{
    background: {COLOR_CONTROL};
    border-radius: 3px;
    min-height: 20px;
    min-width: 20px;
}}
QScrollBar::add-line, QScrollBar::sub-line {{
    background: none;
    border: none;
}}
"""


def _build_palette() -> QPalette:
    palette = QPalette()
    window = QColor(COLOR_WINDOW)
    frame = QColor(COLOR_FRAME)
    control = QColor(COLOR_CONTROL)
    label = QColor(COLOR_LABEL)
    control_text = QColor(COLOR_CONTROL_TEXT)
    highlight = QColor(COLOR_HIGHLIGHT)
    disabled = QColor(COLOR_DISABLED)

    palette.setColor(QPalette.ColorRole.Window, window)
    palette.setColor(QPalette.ColorRole.WindowText, label)
    palette.setColor(QPalette.ColorRole.Base, control)
    palette.setColor(QPalette.ColorRole.AlternateBase, frame)
    palette.setColor(QPalette.ColorRole.Text, control_text)
    palette.setColor(QPalette.ColorRole.PlaceholderText, QColor("#cc7a38"))
    palette.setColor(QPalette.ColorRole.Button, control)
    palette.setColor(QPalette.ColorRole.ButtonText, control_text)
    palette.setColor(QPalette.ColorRole.BrightText, label)
    palette.setColor(QPalette.ColorRole.ToolTipBase, frame)
    palette.setColor(QPalette.ColorRole.ToolTipText, label)
    palette.setColor(QPalette.ColorRole.Highlight, highlight)
    palette.setColor(QPalette.ColorRole.HighlightedText, label)
    palette.setColor(QPalette.ColorRole.Light, QColor(COLOR_CONTROL_HOVER))
    palette.setColor(QPalette.ColorRole.Midlight, control)
    palette.setColor(QPalette.ColorRole.Mid, frame)
    palette.setColor(QPalette.ColorRole.Dark, window)
    palette.setColor(QPalette.ColorRole.Shadow, QColor("#000000"))
    palette.setColor(QPalette.ColorRole.Link, control_text)
    palette.setColor(QPalette.ColorRole.LinkVisited, QColor("#e07018"))

    disabled_group = QPalette.ColorGroup.Disabled
    palette.setColor(disabled_group, QPalette.ColorRole.WindowText, disabled)
    palette.setColor(disabled_group, QPalette.ColorRole.Text, disabled)
    palette.setColor(disabled_group, QPalette.ColorRole.ButtonText, disabled)
    palette.setColor(disabled_group, QPalette.ColorRole.Base, frame)
    palette.setColor(disabled_group, QPalette.ColorRole.Button, frame)
    return palette


def apply_theme(app: QApplication) -> None:
    """Applique le thème rouge / jaune / orange à toute l'application."""
    app.setStyle("Fusion")
    app.setPalette(_build_palette())
    app.setStyleSheet(_STYLESHEET)
