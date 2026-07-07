#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-05    *
#***********************************************
from __future__ import annotations

import cupy as cp


def phase_correlation_shift(reference: cp.ndarray, frame: cp.ndarray) -> tuple[float, float]:
    """Retourne le décalage (dy, dx) pour aligner frame sur reference."""
    ref = reference.astype(cp.float32)
    img = frame.astype(cp.float32)

    ref -= ref.mean()
    img -= img.mean()

    f_ref = cp.fft.fft2(ref)
    f_img = cp.fft.fft2(img)

    cross_power = f_ref * cp.conj(f_img)
    cross_power /= cp.maximum(cp.abs(cross_power), 1e-8)

    correlation = cp.fft.ifft2(cross_power).real
    h, w = reference.shape
    peak_flat = int(cp.argmax(correlation))
    dy, dx = divmod(peak_flat, w)

    if dy > h // 2:
        dy -= h
    if dx > w // 2:
        dx -= w

    sub_dy, sub_dx = _subpixel_offset(correlation, dy, dx)
    return sub_dy, sub_dx


def _subpixel_offset(correlation: cp.ndarray, y: int, x: int) -> tuple[float, float]:
    """Affine le pic de corrélation par interpolation parabolique."""
    h, w = correlation.shape
    dy = float(y)
    dx = float(x)

    if 0 < y < h - 1:
        c_m = float(correlation[y - 1, x])
        c_0 = float(correlation[y, x])
        c_p = float(correlation[y + 1, x])
        denom = c_m - 2.0 * c_0 + c_p
        if abs(denom) > 1e-8:
            dy += 0.5 * (c_m - c_p) / denom

    if 0 < x < w - 1:
        c_m = float(correlation[y, x - 1])
        c_0 = float(correlation[y, x])
        c_p = float(correlation[y, x + 1])
        denom = c_m - 2.0 * c_0 + c_p
        if abs(denom) > 1e-8:
            dx += 0.5 * (c_m - c_p) / denom

    return dy, dx


def compute_shifts(reference: cp.ndarray, frames: cp.ndarray) -> cp.ndarray:
    """Calcule les décalages sous-pixel de chaque frame par rapport à la référence."""
    shifts = cp.zeros((frames.shape[0], 2), dtype=cp.float32)
    for index in range(1, frames.shape[0]):
        dy, dx = phase_correlation_shift(reference, frames[index])
        shifts[index, 0] = dy
        shifts[index, 1] = dx
    return shifts


def shift_image(image: cp.ndarray, dy: float, dx: float) -> cp.ndarray:
    return cp.roll(cp.roll(image, int(round(dy)), axis=0), int(round(dx)), axis=1)


def align_frames(reference: cp.ndarray, frames: cp.ndarray) -> cp.ndarray:
    aligned = cp.empty_like(frames)
    for i in range(frames.shape[0]):
        if i == 0:
            dy, dx = 0.0, 0.0
        else:
            dy, dx = phase_correlation_shift(reference, frames[i])
        aligned[i] = shift_image(frames[i], dy, dx)
    return aligned
