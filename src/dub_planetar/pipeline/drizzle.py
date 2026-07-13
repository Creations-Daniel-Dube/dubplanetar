#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-12    *
#***********************************************
from __future__ import annotations

import cupy as cp
from cupyx.scipy.ndimage import map_coordinates

from dub_planetar.i18n import PipelineError
from dub_planetar.pipeline.debayer import demosaic_bayer


def _output_base_coords(oh: int, ow: int, scale: int) -> tuple[cp.ndarray, cp.ndarray]:
    yo, xo = cp.mgrid[0:oh, 0:ow]
    return (yo.astype(cp.float32) / scale, xo.astype(cp.float32) / scale)


def bayer_superres_stack(
    frames: cp.ndarray,
    shifts: cp.ndarray,
    weights: cp.ndarray,
    *,
    scale: int = 3,
    bayer_pattern: str = "BGGR",
) -> cp.ndarray:
    """Empilement couleur super-résolu (debayer dense + shift-and-add sous-pixel).

    Chaque frame est débayerisée en RGB dense, puis rééchantillonnée sur une
    grille `scale` fois plus fine avec son décalage sous-pixel, et accumulée.
    Retourne une image RGB float32 (h*scale, w*scale, 3).
    """
    if scale < 1:
        raise PipelineError("error.scale_min")

    n, h, w = frames.shape
    oh, ow = h * scale, w * scale
    accum = cp.zeros((oh, ow, 3), dtype=cp.float32)

    base_y, base_x = _output_base_coords(oh, ow, scale)
    w_norm = weights.astype(cp.float32)
    w_norm /= cp.maximum(w_norm.sum(), 1e-8)

    for index in range(n):
        rgb = demosaic_bayer(frames[index], pattern=bayer_pattern)
        dy = float(shifts[index, 0])
        dx = float(shifts[index, 1])
        fw = float(w_norm[index])

        coords = cp.stack([(base_y - dy).ravel(), (base_x - dx).ravel()])
        for channel in range(3):
            sampled = map_coordinates(
                rgb[..., channel], coords, order=1, mode="constant", cval=0.0
            )
            accum[..., channel] += sampled.reshape(oh, ow) * fw

    return accum


def superres_stack_mono(
    frames: cp.ndarray,
    shifts: cp.ndarray,
    weights: cp.ndarray,
    *,
    scale: int = 3,
) -> cp.ndarray:
    """Version niveaux de gris (RAW brut) du shift-and-add super-résolu."""
    if scale < 1:
        raise PipelineError("error.scale_min")

    n, h, w = frames.shape
    oh, ow = h * scale, w * scale
    accum = cp.zeros((oh, ow), dtype=cp.float32)

    base_y, base_x = _output_base_coords(oh, ow, scale)
    w_norm = weights.astype(cp.float32)
    w_norm /= cp.maximum(w_norm.sum(), 1e-8)

    for index in range(n):
        dy = float(shifts[index, 0])
        dx = float(shifts[index, 1])
        fw = float(w_norm[index])

        coords = cp.stack([(base_y - dy).ravel(), (base_x - dx).ravel()])
        sampled = map_coordinates(
            frames[index].astype(cp.float32), coords, order=1, mode="constant", cval=0.0
        )
        accum += sampled.reshape(oh, ow) * fw

    return accum
