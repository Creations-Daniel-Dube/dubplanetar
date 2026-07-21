#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import QObject, Signal

from dub_planetar.i18n import PipelineError
from dub_planetar.pipeline.stacker import StackSettings, stack_video


class StackWorker(QObject):
    progress = Signal(str, float)
    finished = Signal(object)
    failed = Signal(object)

    def __init__(
        self,
        input_path: Path,
        output_path: Path,
        settings: StackSettings,
    ) -> None:
        super().__init__()
        self.input_path = input_path
        self.output_path = output_path
        self.settings = settings

    def run(self) -> None:
        try:
            result = stack_video(
                self.input_path,
                self.output_path,
                self.settings,
                on_progress=self._on_progress,
            )
        except PipelineError as exc:
            self.failed.emit(exc)
            return
        except Exception as exc:  # noqa: BLE001
            self.failed.emit(str(exc))
            return

        self.finished.emit(result)

    def _on_progress(self, stage: str, value: float) -> None:
        self.progress.emit(stage, value)
