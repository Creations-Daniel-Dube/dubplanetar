#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
from __future__ import annotations

import math

import cupy as cp
from cupyx.scipy.ndimage import gaussian_filter


def laplacian_variance_scores(frames: cp.ndarray) -> cp.ndarray:
    """Score de netteté par frame, centré sur la SURFACE du disque.

    On mesure l'énergie des hautes fréquences (détail de surface) uniquement à
    l'intérieur du disque, en excluant le limbe : le bord disque→ciel est
    identique sur toutes les frames et masquerait la vraie netteté de surface.
    """
    scores = cp.empty(frames.shape[0], dtype=cp.float32)
    for i in range(frames.shape[0]):
        frame = frames[i].astype(cp.float32)
        interior = _interior_mask(frame)
        highpass = frame - gaussian_filter(frame, sigma=1.5)
        if int(interior.sum()) > 16:
            vals = highpass[interior]
            scores[i] = vals.var()
        else:
            scores[i] = highpass.var()

    return scores


def _interior_mask(frame: cp.ndarray) -> cp.ndarray:
    """Masque de l'intérieur du disque, limbe exclu (fortement flouté)."""
    smooth = gaussian_filter(frame, sigma=6.0)
    peak = float(smooth.max())
    if peak <= 0:
        return cp.zeros(frame.shape, dtype=bool)
    return smooth > peak * 0.6


def _convolve2d(image: cp.ndarray, kernel: cp.ndarray) -> cp.ndarray:
    kh, kw = kernel.shape
    pad_h, pad_w = kh // 2, kw // 2
    padded = cp.pad(image, ((pad_h, pad_h), (pad_w, pad_w)), mode="reflect")
    out = cp.zeros_like(image)

    for y in range(kernel.shape[0]):
        for x in range(kernel.shape[1]):
            out += kernel[y, x] * padded[y : y + image.shape[0], x : x + image.shape[1]]

    return out


def select_top_frames(
    scores: cp.ndarray,
    *,
    keep_ratio: float,
    min_frames: int = 3,
) -> cp.ndarray:
    keep_ratio = min(1.0, max(0.05, float(keep_ratio)))
    count = max(min_frames, math.ceil(scores.size * keep_ratio))
    count = min(count, int(scores.size))
    order = cp.argsort(scores)[::-1]
    return order[:count]
