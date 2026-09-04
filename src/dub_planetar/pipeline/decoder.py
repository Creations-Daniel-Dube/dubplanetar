#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-09-03    *
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


@dataclass(frozen=True)
class VideoProbe:
    frame_count: int
    fps: float
    width: int
    height: int


def probe_avi(path: Path) -> VideoProbe:
    """Lit les métadonnées d'un AVI sans charger les frames."""
    capture = cv2.VideoCapture(str(path))
    if not capture.isOpened():
        raise PipelineError("error.video_open", path)
    try:
        return VideoProbe(
            frame_count=max(0, int(capture.get(cv2.CAP_PROP_FRAME_COUNT)) or 0),
            fps=float(capture.get(cv2.CAP_PROP_FPS) or 0.0),
            width=int(capture.get(cv2.CAP_PROP_FRAME_WIDTH) or 0),
            height=int(capture.get(cv2.CAP_PROP_FRAME_HEIGHT) or 0),
        )
    finally:
        capture.release()


def open_avi_capture(path: Path) -> cv2.VideoCapture:
    capture = cv2.VideoCapture(str(path))
    if not capture.isOpened():
        raise PipelineError("error.video_open", path)
    return capture


def _normalize_raw(raw: np.ndarray) -> np.ndarray:
    if raw.dtype == np.uint16:
        return raw.astype(np.float32)
    return raw.astype(np.float32) * (65535.0 / 255.0)


def _seek_frame(capture: cv2.VideoCapture, index: int) -> None:
    capture.set(cv2.CAP_PROP_POS_FRAMES, max(0, index))


def read_capture_frame(capture: cv2.VideoCapture, index: int) -> np.ndarray | None:
    """Lit une frame (plan RAW normalisé float32) à l'index donné."""
    _seek_frame(capture, max(0, index))
    ok, frame = capture.read()
    if not ok:
        return None
    return _normalize_raw(_extract_raw_plane(frame))


def iter_avi_frames(
    path: Path,
    *,
    max_frames: int | None = None,
    start_frame: int = 0,
    end_frame: int | None = None,
    on_progress: ProgressCallback | None = None,
) -> tuple[int, int, list[np.ndarray]]:
    """Lit les frames d'un AVI RAW SeeStar (mono 8/16 bits)."""
    capture = cv2.VideoCapture(str(path))
    if not capture.isOpened():
        raise PipelineError("error.video_open", path)

    total = int(capture.get(cv2.CAP_PROP_FRAME_COUNT)) or 0
    width = int(capture.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(capture.get(cv2.CAP_PROP_FRAME_HEIGHT))

    start = max(0, start_frame)
    last = end_frame
    if last is not None:
        last = max(start, last)
    if total > 0 and last is not None:
        last = min(last, total - 1)

    span = 1
    if last is not None:
        span = max(1, last - start + 1)
    elif total > start:
        span = total - start

    frames: list[np.ndarray] = []
    index = start
    if start > 0:
        _seek_frame(capture, start)

    while True:
        if last is not None and index > last:
            break

        ok, frame = capture.read()
        if not ok:
            break

        frames.append(_normalize_raw(_extract_raw_plane(frame)))
        index += 1

        if max_frames is not None and len(frames) >= max_frames:
            break

        if on_progress and span > 0 and len(frames) % 25 == 0:
            on_progress("stage.read_frames", min(1.0, len(frames) / span))

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
