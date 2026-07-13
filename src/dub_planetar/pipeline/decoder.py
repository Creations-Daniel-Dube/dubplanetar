#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-12    *
#***********************************************
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable

import cv2
import numpy as np

from dub_planetar.i18n import PipelineError


@dataclass(frozen=True)
class FrameBatch:
    indices: np.ndarray
    frames: np.ndarray  # (N, H, W) float32


ProgressCallback = Callable[[str, float], None]


def iter_avi_frames(
    path: Path,
    *,
    max_frames: int | None = None,
    on_progress: ProgressCallback | None = None,
) -> tuple[int, int, list[np.ndarray]]:
    """Lit toutes les frames d'un AVI RAW SeeStar (mono 8/16 bits)."""
    capture = cv2.VideoCapture(str(path))
    if not capture.isOpened():
        raise PipelineError("error.video_open", path)

    total = int(capture.get(cv2.CAP_PROP_FRAME_COUNT)) or 0
    width = int(capture.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(capture.get(cv2.CAP_PROP_FRAME_HEIGHT))

    frames: list[np.ndarray] = []
    index = 0

    while True:
        ok, frame = capture.read()
        if not ok:
            break

        raw = _extract_raw_plane(frame)
        if raw.dtype == np.uint16:
            normalized = raw.astype(np.float32)
        else:
            normalized = raw.astype(np.float32) * (65535.0 / 255.0)

        frames.append(normalized)
        index += 1

        if max_frames is not None and index >= max_frames:
            break

        if on_progress and total > 0 and index % 25 == 0:
            on_progress("stage.read_frames", index / total)

    capture.release()

    if not frames:
        raise PipelineError("error.no_frames")

    if on_progress:
        on_progress("stage.read_done", 1.0)

    return width, height, frames


def _extract_raw_plane(frame: np.ndarray) -> np.ndarray:
    """Extrait le plan RAW mono sans mélanger les canaux couleur."""
    if frame.ndim == 2:
        return frame

    if frame.ndim == 3:
        # SeeStar RAW : souvent 3 plans identiques ou un seul plan utile
        if frame.shape[2] >= 3:
            b, g, r = frame[..., 0], frame[..., 1], frame[..., 2]
            if np.array_equal(b, g) and np.array_equal(g, r):
                return b
            # Si les canaux diffèrent, c'est déjà du RGB — prendre la luminance
            # préserve au moins la structure sans moyenne destructive.
            return cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        return frame[..., 0]

    raise PipelineError("error.unsupported_frame", frame.shape)


def frames_to_batch(frames: list[np.ndarray]) -> FrameBatch:
    stacked = np.stack(frames, axis=0)
    indices = np.arange(len(frames), dtype=np.int32)
    return FrameBatch(indices=indices, frames=stacked)
