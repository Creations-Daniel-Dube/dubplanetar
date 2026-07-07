#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-05    *
#***********************************************
from __future__ import annotations

import cv2
import cupy as cp
import numpy as np

from dub_planetar.i18n import PipelineError

_BAYER_CODES: dict[str, int] = {
    "BGGR": cv2.COLOR_BAYER_BG2RGB,
    "RGGB": cv2.COLOR_BAYER_RG2RGB,
    "GRBG": cv2.COLOR_BAYER_GR2RGB,
    "GBRG": cv2.COLOR_BAYER_GB2RGB,
}

# Alias courant (SeeStar) — « GRGB » = GRBG en notation standard
_BAYER_ALIASES: dict[str, str] = {
    "GRGB": "GRBG",
    "AUTO": "AUTO",
}

# Ordre de recherche auto : motifs qui marchent le mieux sur SeeStar en premier
_AUTO_CANDIDATES = ("GRBG", "GBRG", "BGGR", "RGGB")


def normalize_pattern(pattern: str) -> str:
    key = pattern.upper().strip()
    return _BAYER_ALIASES.get(key, key)


def demosaic_bayer(raw: cp.ndarray, *, pattern: str = "GRBG") -> cp.ndarray:
    """Debayer bilinéaire via OpenCV (fiable, sans trame résiduelle)."""
    resolved = normalize_pattern(pattern)
    if resolved == "AUTO":
        resolved = detect_bayer_pattern(raw)

    code = _BAYER_CODES.get(resolved)
    if code is None:
        raise PipelineError("error.unknown_bayer", pattern)

    arr = cp.asnumpy(raw)
    if arr.ndim != 2:
        raise PipelineError("error.debayer_mono")

    # OpenCV attend uint8 ou uint16
    if arr.dtype == np.uint16:
        bayer = arr
    else:
        bayer = np.clip(arr, 0, 65535).astype(np.uint16)

    rgb = cv2.cvtColor(bayer, code)
    return cp.asarray(rgb.astype(np.float32))


def detect_bayer_pattern(raw: cp.ndarray) -> str:
    """Choisit le motif Bayer qui minimise les artefacts couleur sur le disque."""
    best_pattern = "GRBG"
    best_score = float("inf")

    for pattern in _AUTO_CANDIDATES:
        rgb = demosaic_bayer(raw, pattern=pattern)
        score = _debayer_quality_score(rgb)
        if score < best_score:
            best_score = score
            best_pattern = pattern

    return best_pattern


def _debayer_quality_score(rgb: cp.ndarray) -> float:
    """Score bas : disque neutre, peu de franges et pas de grille résiduelle."""
    lum = rgb.mean(axis=2)
    peak = float(lum.max())
    if peak <= 0:
        return float("inf")

    mask = lum > peak * 0.12
    if int(mask.sum()) < 64:
        mask = lum > peak * 0.05
    if int(mask.sum()) < 16:
        return float("inf")

    red = rgb[..., 0]
    green = rgb[..., 1]
    blue = rgb[..., 2]
    signal = max(float(lum[mask].mean()), 1.0)

    chroma = float(cp.abs(red - green)[mask].mean() + cp.abs(green - blue)[mask].mean()) / signal

    diff_rg = cp.abs(red - green)
    lap = diff_rg[1:-1, 1:-1] - 0.25 * (
        diff_rg[:-2, 1:-1]
        + diff_rg[2:, 1:-1]
        + diff_rg[1:-1, :-2]
        + diff_rg[1:-1, 2:]
    )
    inner_mask = mask[1:-1, 1:-1]
    grid = float(cp.abs(lap)[inner_mask].mean()) / signal if int(inner_mask.sum()) else 0.0

    means = cp.array([red[mask].mean(), green[mask].mean(), blue[mask].mean()])
    cast = float(cp.std(means)) / max(float(cp.mean(means)), 1.0)

    return chroma + grid * 2.0 + cast * 0.5


def gray_world_white_balance(rgb: cp.ndarray) -> cp.ndarray:
    """Égalise la moyenne des 3 canaux (hypothèse gray-world)."""
    luminance = rgb.mean(axis=2)
    threshold = float(luminance.max()) * 0.05
    mask = luminance > threshold
    if int(mask.sum()) < 16:
        mask = luminance > 0

    means = cp.stack([rgb[..., c][mask].mean() for c in range(3)])
    means = cp.maximum(means, 1e-6)
    target = float(means.mean())
    gains = target / means

    return rgb * gains.reshape(1, 1, 3)
