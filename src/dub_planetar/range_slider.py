#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-09-10    *
#***********************************************
from __future__ import annotations

from PySide6.QtCore import QPoint, QRect, Qt, Signal
from PySide6.QtGui import QColor, QMouseEvent, QPainter, QPaintEvent, QPen, QPolygon
from PySide6.QtWidgets import QSizePolicy, QWidget

from dub_planetar.theme import (
    COLOR_BORDER,
    COLOR_CONTROL,
    COLOR_DISABLED,
    COLOR_FRAME,
    COLOR_HIGHLIGHT,
    COLOR_LABEL,
)

_HANDLE_W = 12
_HANDLE_H = 16
_GROOVE_H = 8
_PAD_X = 8


class RangeSlider(QWidget):
    """Timeline à deux poignées (début / fin) sur une seule piste."""

    startChanged = Signal(int)
    endChanged = Signal(int)
    handleMoved = Signal(str, int)
    sliderReleased = Signal(str, int)

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._minimum = 0
        self._maximum = 0
        self._start = 0
        self._end = 0
        self._active: str | None = None
        self.setMinimumHeight(28)
        self.setMinimumWidth(160)
        self.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)
        self.setMouseTracking(True)
        self.setCursor(Qt.CursorShape.PointingHandCursor)

    def set_span(self, minimum: int, maximum: int) -> None:
        minimum = max(0, int(minimum))
        maximum = max(minimum, int(maximum))
        self._minimum = minimum
        self._maximum = maximum
        self._start = minimum
        self._end = maximum
        self.update()

    def reset_full_span(self) -> None:
        self._start = self._minimum
        self._end = self._maximum
        self.update()

    def start(self) -> int:
        return self._start

    def end(self) -> int:
        return self._end

    def _span(self) -> int:
        return max(1, self._maximum - self._minimum)

    def _groove_rect(self) -> QRect:
        y = (self.height() - _GROOVE_H) // 2
        return QRect(_PAD_X, y, max(1, self.width() - 2 * _PAD_X), _GROOVE_H)

    def _value_to_x(self, value: int) -> int:
        groove = self._groove_rect()
        if self._maximum == self._minimum:
            return groove.left()
        ratio = (value - self._minimum) / self._span()
        return int(groove.left() + ratio * groove.width())

    def _x_to_value(self, x: int) -> int:
        groove = self._groove_rect()
        if groove.width() <= 0 or self._maximum == self._minimum:
            return self._minimum
        ratio = (x - groove.left()) / groove.width()
        value = self._minimum + round(ratio * self._span())
        return max(self._minimum, min(self._maximum, value))

    def _handle_rect(self, value: int) -> QRect:
        cx = self._value_to_x(value)
        return QRect(
            cx - _HANDLE_W // 2,
            (self.height() - _HANDLE_H) // 2,
            _HANDLE_W,
            _HANDLE_H,
        )

    def _hit_handle(self, pos: QPoint) -> str | None:
        start_rect = self._handle_rect(self._start).adjusted(-4, -4, 4, 4)
        end_rect = self._handle_rect(self._end).adjusted(-4, -4, 4, 4)
        in_start = start_rect.contains(pos)
        in_end = end_rect.contains(pos)
        if in_start and in_end:
            start_dist = abs(pos.x() - start_rect.center().x())
            end_dist = abs(pos.x() - end_rect.center().x())
            return "start" if start_dist <= end_dist else "end"
        if in_start:
            return "start"
        if in_end:
            return "end"
        return None

    def _apply_value(self, which: str, value: int) -> bool:
        if which == "start":
            value = max(self._minimum, min(value, self._end))
            if value == self._start:
                return False
            self._start = value
            self.startChanged.emit(self._start)
            return True
        value = min(self._maximum, max(value, self._start))
        if value == self._end:
            return False
        self._end = value
        self.endChanged.emit(self._end)
        return True

    def mousePressEvent(self, event: QMouseEvent) -> None:
        if event.button() != Qt.MouseButton.LeftButton or not self.isEnabled():
            return
        which = self._hit_handle(event.position().toPoint())
        if which is None:
            value = self._x_to_value(event.position().toPoint().x())
            dist_start = abs(value - self._start)
            dist_end = abs(value - self._end)
            which = "start" if dist_start <= dist_end else "end"
        self._active = which
        if self._apply_value(which, self._x_to_value(event.position().toPoint().x())):
            self.handleMoved.emit(which, self._start if which == "start" else self._end)
            self.update()

    def mouseMoveEvent(self, event: QMouseEvent) -> None:
        if self._active is None or not self.isEnabled():
            return
        which = self._active
        if self._apply_value(which, self._x_to_value(event.position().toPoint().x())):
            self.handleMoved.emit(which, self._start if which == "start" else self._end)
            self.update()

    def mouseReleaseEvent(self, event: QMouseEvent) -> None:
        if event.button() != Qt.MouseButton.LeftButton or self._active is None:
            return
        which = self._active
        self._active = None
        self.sliderReleased.emit(which, self._start if which == "start" else self._end)

    def paintEvent(self, event: QPaintEvent) -> None:  # noqa: ARG002
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        enabled = self.isEnabled()
        groove_bg = QColor(COLOR_FRAME) if enabled else QColor(COLOR_CONTROL)
        selected = QColor(COLOR_HIGHLIGHT) if enabled else QColor(COLOR_DISABLED)
        handle_fill = QColor(COLOR_LABEL) if enabled else QColor(COLOR_DISABLED)
        handle_border = QColor(COLOR_BORDER) if enabled else QColor(COLOR_FRAME)

        groove = self._groove_rect()
        painter.setPen(Qt.PenStyle.NoPen)
        painter.setBrush(groove_bg)
        painter.drawRoundedRect(groove, 3, 3)

        x0 = self._value_to_x(self._start)
        x1 = self._value_to_x(self._end)
        selected_rect = QRect(
            min(x0, x1),
            groove.top(),
            max(1, abs(x1 - x0)),
            groove.height(),
        )
        painter.setBrush(selected)
        painter.drawRoundedRect(selected_rect, 3, 3)

        painter.setPen(QPen(handle_border, 1))
        painter.setBrush(handle_fill)
        for value in (self._start, self._end):
            rect = self._handle_rect(value)
            triangle = QPolygon(
                [
                    QPoint(rect.center().x(), rect.top()),
                    QPoint(rect.right(), rect.bottom()),
                    QPoint(rect.left(), rect.bottom()),
                ]
            )
            painter.drawPolygon(triangle)
