#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-21    *
#***********************************************
from __future__ import annotations

import cupy as cp
from cupyx.scipy.ndimage import gaussian_filter


def weighted_average_stack(frames: cp.ndarray, weights: cp.ndarray) -> cp.ndarray:
    """Empilement par moyenne pondérée (poids = scores de netteté)."""
    w = weights.astype(cp.float32)
    w = w / cp.maximum(w.sum(), 1e-8)
    stacked = cp.tensordot(w, frames, axes=(0, 0))
    return stacked.astype(cp.float32)


def _apply_gamma(stretched: cp.ndarray, gamma: float) -> cp.ndarray:
    if abs(gamma - 1.0) > 1e-3:
        return stretched ** (1.0 / gamma)
    return stretched


def _corner_pixels(channel: cp.ndarray, *, corner_frac: float = 0.14) -> cp.ndarray:
    h, w = channel.shape
    ch = max(1, int(h * corner_frac))
    cw = max(1, int(w * corner_frac))
    return cp.concatenate(
        [
            channel[:ch, :cw].ravel(),
            channel[:ch, -cw:].ravel(),
            channel[-ch:, :cw].ravel(),
            channel[-ch:, -cw:].ravel(),
        ]
    )


def estimate_sky_background(luminance: cp.ndarray, *, corner_frac: float = 0.14) -> float:
    """Estime le niveau du fond de ciel (luminance) depuis les 4 coins."""
    corners = _corner_pixels(luminance, corner_frac=corner_frac)
    return float(cp.percentile(corners, 90))


def estimate_sky_background_rgb(rgb: cp.ndarray, *, corner_frac: float = 0.14) -> cp.ndarray:
    """Estime le fond de ciel par canal R, V, B (évite le voile rouge)."""
    lows = cp.empty(3, dtype=cp.float32)
    for channel in range(3):
        corners = _corner_pixels(rgb[..., channel], corner_frac=corner_frac)
        lows[channel] = cp.percentile(corners, 90)
    return lows


def asinh_compress_rgb(img: cp.ndarray, stretch: float) -> cp.ndarray:
    """Compresse les hautes lumières (arcsinh) en préservant la couleur.

    Normalisée sur le vrai maximum : le centre du disque devient le point blanc
    sans jamais être écrêté, et la plage sous le pic conserve son contraste.
    """
    if stretch <= 0:
        return img

    data = cp.maximum(img.astype(cp.float32), 0.0)
    luminance = data.mean(axis=2, keepdims=True)
    peak = float(luminance.max())
    if peak <= 0:
        return data

    factor = stretch / peak
    compressed = cp.arcsinh(luminance * factor)
    ref = float(cp.arcsinh(peak * factor))
    if ref > 0:
        compressed = compressed / ref

    scale = compressed / cp.maximum(luminance, 1e-6)
    result = data * scale
    return cp.clip(result, 0.0, 1.0)


def wavelet_sharpen(
    image: cp.ndarray,
    *,
    amount: float,
    radius: float,
    levels: int = 4,
) -> cp.ndarray:
    """Accentuation multi-échelle (ondelettes « à trous », façon Registax).

    L'image est décomposée en plusieurs couches de détail (fin → large) via des
    gaussiennes successives. Chaque couche est ré-amplifiée puis recombinée, ce
    qui révèle la granulation, les pénombres et les structures fines sans le
    halo d'un unsharp mono-échelle. Le détail est calculé sur la luminance et
    réinjecté sur les 3 canaux pour éviter les franges de couleur.
    """
    if amount <= 0 or radius <= 0:
        return image

    data = image.astype(cp.float32)
    is_rgb = data.ndim == 3
    luminance = data.mean(axis=2) if is_rgb else data

    residual = luminance
    detail_sum = cp.zeros_like(luminance)
    sigma = radius
    # Les échelles fines sont accentuées plus fort que les échelles larges.
    for level in range(levels):
        blurred = gaussian_filter(residual, sigma=sigma)
        detail = residual - blurred
        weight = amount * (0.6 ** level)
        detail_sum += weight * detail
        residual = blurred
        sigma *= 2.0

    if is_rgb:
        return cp.clip(data + detail_sum[..., None], 0.0, 1.0)
    return cp.clip(data + detail_sum, 0.0, 1.0)


def flatten_illumination(
    linear: cp.ndarray,
    strength: float,
    *,
    scale_frac: float = 0.16,
    base_level: float = 0.55,
) -> cp.ndarray:
    """Uniformise la luminosité du disque et le ramène à un gris moyen.

    Divise l'image par sa version très floutée (modèle d'illumination grande
    échelle), puis recale le disque autour de `base_level`. Le centre trop
    brillant descend au niveau des bords : granulation et taches ressortent
    uniformément, y compris au centre sur-exposé.

    Retourne des valeurs déjà "display-referred" (autour de base_level) : le
    ré-étirement au maximum est court-circuité en aval.
    """
    if strength <= 0:
        return linear

    is_rgb = linear.ndim == 3
    luminance = linear.mean(axis=2) if is_rgb else linear
    peak = float(luminance.max())
    if peak <= 0:
        return linear

    disk = luminance > peak * 0.05
    sigma = max(luminance.shape) * scale_frac
    illum = gaussian_filter(luminance, sigma=sigma)

    target = float(cp.median(illum[disk])) if int(disk.sum()) else float(illum.max())
    if target <= 0:
        return linear

    # gain qui ramène l'illumination locale à base_level (disque → gris moyen)
    denom = cp.maximum(illum, target * 0.15)
    flat_gain = base_level / denom

    # gain "neutre" (étirement global classique) pour pouvoir doser l'effet
    neutral_gain = base_level / target
    gain = neutral_gain + strength * (flat_gain - neutral_gain)
    gain = cp.clip(gain, 0.0, base_level / (target * 0.15))

    if is_rgb:
        return linear * gain[..., None]
    return linear * gain


def _feathered_disk(luminance: cp.ndarray, *, threshold_ratio: float = 0.04) -> cp.ndarray:
    """Masque doux du disque (0 dehors, 1 dedans) pour éviter une ligne noire."""
    peak = float(luminance.max())
    if peak <= 0:
        return cp.ones(luminance.shape, dtype=cp.float32)
    thr = peak * threshold_ratio
    soft = cp.clip((luminance - thr * 0.5) / (thr + 1e-6), 0.0, 1.0)
    return soft.astype(cp.float32)


def _sky_mask(luminance: cp.ndarray, *, threshold_ratio: float = 0.05) -> cp.ndarray:
    peak = float(luminance.max())
    if peak <= 0:
        return cp.zeros(luminance.shape, dtype=bool)
    return luminance > peak * threshold_ratio


def _stretch_channels(
    linear: cp.ndarray,
    luminance: cp.ndarray,
    *,
    white_point: float,
    gamma: float,
    solar_stretch: float,
    prescaled: bool = False,
) -> cp.ndarray:
    if prescaled:
        # Aplatissement déjà appliqué : les valeurs sont calibrées (~gris moyen),
        # on ne redivise pas par le maximum (sinon le disque redevient blanc).
        stretched = cp.clip(linear, 0.0, 1.0)
    else:
        high = float(cp.percentile(luminance, white_point))
        span = max(high, 1.0)
        if solar_stretch > 0:
            base = cp.clip(linear / span, 0.0, 1.0)
            stretched = asinh_compress_rgb(base * 65535.0, solar_stretch)
        else:
            stretched = cp.clip(linear / span, 0.0, 1.0)

    return _apply_gamma(stretched, gamma)


def _finalize(
    stretched: cp.ndarray,
    luminance: cp.ndarray,
    *,
    sharpen_amount: float,
    sharpen_radius: float,
) -> cp.ndarray:
    if sharpen_amount > 0:
        stretched = wavelet_sharpen(
            stretched, amount=sharpen_amount, radius=sharpen_radius
        )

    disk = _feathered_disk(luminance)
    if stretched.ndim == 3:
        stretched = stretched * disk[..., None]
    else:
        stretched = stretched * disk
    return (cp.clip(stretched, 0.0, 1.0) * 65535.0).astype(cp.uint16)


def normalize_to_uint16(
    image: cp.ndarray,
    *,
    black_point: float = 0.1,
    white_point: float = 100.0,
    gamma: float = 1.0,
    subtract_background: bool = True,
    solar_stretch: float = 0.0,
    sharpen_amount: float = 0.0,
    sharpen_radius: float = 2.0,
    flatten_strength: float = 0.0,
) -> cp.ndarray:
    img = image.astype(cp.float32)
    if subtract_background:
        low = estimate_sky_background(img)
        if black_point > 0:
            low = max(low, float(cp.percentile(img, black_point)))
    else:
        low = float(cp.percentile(img, black_point))
    linear = cp.maximum(img - low, 0.0)

    mask_lum = linear
    flattened = flatten_strength > 0
    if flattened:
        linear = flatten_illumination(linear, flatten_strength)

    stretched = _stretch_channels(
        linear,
        linear,
        white_point=white_point,
        gamma=gamma,
        solar_stretch=solar_stretch,
        prescaled=flattened,
    )
    return _finalize(
        stretched,
        mask_lum,
        sharpen_amount=sharpen_amount,
        sharpen_radius=sharpen_radius,
    )


def normalize_rgb_to_uint16(
    rgb: cp.ndarray,
    *,
    black_point: float = 0.1,
    white_point: float = 100.0,
    gamma: float = 1.0,
    subtract_background: bool = True,
    solar_stretch: float = 0.0,
    sharpen_amount: float = 0.0,
    sharpen_radius: float = 2.0,
    flatten_strength: float = 0.0,
) -> cp.ndarray:
    """Étirement tonal RGB + accentuation, fond de ciel noir hors du disque."""
    img = rgb.astype(cp.float32)

    if subtract_background:
        lows = estimate_sky_background_rgb(img)
        if black_point > 0:
            luminance = img.mean(axis=2)
            lum_floor = float(cp.percentile(luminance, black_point))
            lows = cp.maximum(lows, lum_floor)
        linear = cp.maximum(img - lows.reshape(1, 1, 3), 0.0)
    else:
        low = float(cp.percentile(img.mean(axis=2), black_point))
        linear = cp.maximum(img - low, 0.0)

    mask_lum = linear.mean(axis=2)
    flattened = flatten_strength > 0
    if flattened:
        linear = flatten_illumination(linear, flatten_strength)

    luminance = linear.mean(axis=2)
    stretched = _stretch_channels(
        linear,
        luminance,
        white_point=white_point,
        gamma=gamma,
        solar_stretch=solar_stretch,
        prescaled=flattened,
    )
    return _finalize(
        stretched,
        mask_lum,
        sharpen_amount=sharpen_amount,
        sharpen_radius=sharpen_radius,
    )
