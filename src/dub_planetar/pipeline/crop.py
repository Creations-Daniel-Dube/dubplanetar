#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-05    *
#***********************************************
from __future__ import annotations

from dataclasses import dataclass

import cupy as cp


@dataclass(frozen=True)
class CropBox:
    y0: int
    y1: int
    x0: int
    x1: int

    @property
    def height(self) -> int:
        return self.y1 - self.y0

    @property
    def width(self) -> int:
        return self.x1 - self.x0


def detect_disc_crop(
    image: cp.ndarray,
    *,
    margin_ratio: float = 0.05,
    threshold_ratio: float = 0.08,
) -> CropBox:
    """Détecte le disque (Soleil/Lune) et retourne un carré serré avec marge."""
    h, w = image.shape
    lum = image.astype(cp.float32)
    peak = float(lum.max())
    if peak <= 0:
        return CropBox(0, h, 0, w)

    mask = lum > peak * threshold_ratio
    if int(mask.sum()) < 16:
        return CropBox(0, h, 0, w)

    ys, xs = cp.where(mask)
    cy = float(ys.mean())
    cx = float(xs.mean())
    distances = cp.sqrt((ys.astype(cp.float32) - cy) ** 2 + (xs.astype(cp.float32) - cx) ** 2)
    radius = float(cp.percentile(distances, 98))

    margin = radius * margin_ratio
    side = (radius + margin) * 2.0

    y0 = int(max(0, cp.floor(cy - side * 0.5)))
    y1 = int(min(h, cp.ceil(cy + side * 0.5)))
    x0 = int(max(0, cp.floor(cx - side * 0.5)))
    x1 = int(min(w, cp.ceil(cx + side * 0.5)))

    return _snap_even_crop(CropBox(y0, y1, x0, x1), h, w)


def crop_image(image: cp.ndarray, box: CropBox) -> cp.ndarray:
    return image[box.y0 : box.y1, box.x0 : box.x1]


def crop_frames(frames: cp.ndarray, box: CropBox) -> cp.ndarray:
    return frames[:, box.y0 : box.y1, box.x0 : box.x1]


def _snap_even_crop(box: CropBox, max_h: int, max_w: int) -> CropBox:
    """Aligne le recadrage sur la grille Bayer (coordonnées paires)."""
    y0 = box.y0 - (box.y0 % 2)
    x0 = box.x0 - (box.x0 % 2)
    y1 = min(max_h, box.y1 + (box.y1 % 2))
    x1 = min(max_w, box.x1 + (box.x1 % 2))

    if (y1 - y0) % 2:
        y1 -= 1
    if (x1 - x0) % 2:
        x1 -= 1

    if y1 <= y0 or x1 <= x0:
        return CropBox(0, max_h - (max_h % 2), 0, max_w - (max_w % 2))

    return CropBox(y0, y1, x0, x1)
