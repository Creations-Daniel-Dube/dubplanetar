#***********************************************
#* (c) Créations Daniel Dubé     Daniel Dubé   *
#* Dernières Modifications -->   2026-07-12    *
#***********************************************
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import cupy as cp
import numpy as np
import tifffile

from dub_planetar.pipeline.alignment import compute_shifts
from dub_planetar.pipeline.crop import crop_frames, detect_disc_crop
from dub_planetar.pipeline.debayer import detect_bayer_pattern, gray_world_white_balance, normalize_pattern
from dub_planetar.pipeline.decoder import ProgressCallback, frames_to_batch, iter_avi_frames
from dub_planetar.pipeline.drizzle import bayer_superres_stack, superres_stack_mono
from dub_planetar.pipeline.scoring import laplacian_variance_scores, select_top_frames
from dub_planetar.pipeline.stacking import normalize_rgb_to_uint16, normalize_to_uint16


@dataclass(frozen=True)
class StackSettings:
    keep_ratio: float = 0.5
    debayer: bool = True
    white_balance: bool = True
    auto_crop: bool = True
    crop_margin_ratio: float = 0.05
    drizzle_scale: int = 3
    bayer_pattern: str = "AUTO"
    black_point: float = 0.1
    white_point: float = 100.0
    gamma: float = 1.2
    subtract_background: bool = True
    solar_stretch: float = 0.0
    sharpen_amount: float = 1.5
    sharpen_radius: float = 2.0
    flatten_strength: float = 0.0
    max_frames: int | None = None


@dataclass(frozen=True)
class StackResult:
    output_path: Path
    frame_count: int
    kept_frames: int
    width: int
    height: int
    rgb: bool
    bayer_pattern: str | None = None


def stack_video(
    input_path: Path,
    output_path: Path,
    settings: StackSettings,
    *,
    on_progress: ProgressCallback | None = None,
) -> StackResult:
    def report(stage: str, value: float) -> None:
        if on_progress:
            on_progress(stage, value)

    report("stage.read_avi", 0.0)
    width, height, frames = iter_avi_frames(
        input_path,
        max_frames=settings.max_frames,
        on_progress=lambda msg, frac: report(msg, frac * 0.25),
    )

    report("stage.gpu_transfer", 0.25)
    batch = frames_to_batch(frames)
    del frames

    gpu_frames = cp.asarray(batch.frames)
    report("stage.sharpness", 0.35)
    scores = laplacian_variance_scores(gpu_frames)

    report("stage.frame_select", 0.45)
    selected = select_top_frames(scores, keep_ratio=settings.keep_ratio)
    selected_frames = gpu_frames[selected]
    selected_scores = scores[selected]

    report("stage.align", 0.55)
    reference = selected_frames[0]
    shifts = compute_shifts(reference, selected_frames)

    report("stage.crop", 0.62)
    if settings.auto_crop:
        crop_box = detect_disc_crop(reference, margin_ratio=settings.crop_margin_ratio)
        working_frames = crop_frames(selected_frames, crop_box)
    else:
        working_frames = selected_frames

    report("stage.superres", 0.72)
    resolved_bayer: str | None = None
    if settings.debayer:
        pattern_key = normalize_pattern(settings.bayer_pattern)
        if pattern_key == "AUTO":
            report("stage.bayer_detect", 0.68)
            resolved_bayer = detect_bayer_pattern(working_frames[0])
        else:
            resolved_bayer = pattern_key

        rgb_linear = bayer_superres_stack(
            working_frames,
            shifts,
            selected_scores,
            scale=settings.drizzle_scale,
            bayer_pattern=resolved_bayer,
        )
        if settings.white_balance:
            rgb_linear = gray_world_white_balance(rgb_linear)
        rgb = normalize_rgb_to_uint16(
            rgb_linear,
            black_point=settings.black_point,
            white_point=settings.white_point,
            gamma=settings.gamma,
            subtract_background=settings.subtract_background,
            solar_stretch=settings.solar_stretch,
            sharpen_amount=settings.sharpen_amount,
            sharpen_radius=settings.sharpen_radius,
            flatten_strength=settings.flatten_strength,
        )
        output = cp.asnumpy(rgb)
        rgb_output = True
        out_h, out_w = output.shape[:2]
    else:
        mono_linear = superres_stack_mono(
            working_frames,
            shifts,
            selected_scores,
            scale=settings.drizzle_scale,
        )
        mono = normalize_to_uint16(
            mono_linear,
            black_point=settings.black_point,
            white_point=settings.white_point,
            gamma=settings.gamma,
            subtract_background=settings.subtract_background,
            solar_stretch=settings.solar_stretch,
            sharpen_amount=settings.sharpen_amount,
            sharpen_radius=settings.sharpen_radius,
            flatten_strength=settings.flatten_strength,
        )
        output = cp.asnumpy(mono)
        rgb_output = False
        out_h, out_w = output.shape

    output_path.parent.mkdir(parents=True, exist_ok=True)
    tifffile.imwrite(str(output_path), output)

    report("stage.done", 1.0)
    return StackResult(
        output_path=output_path,
        frame_count=int(batch.frames.shape[0]),
        kept_frames=int(selected.size),
        width=out_w,
        height=out_h,
        rgb=rgb_output,
        bayer_pattern=resolved_bayer,
    )


def check_cuda_available() -> tuple[str, tuple[object, ...]]:
    try:
        device = cp.cuda.Device(0)
        device.use()
        name = cp.cuda.runtime.getDeviceProperties(0)["name"].decode()
        mem = device.mem_info[1] / (1024**3)
        return "gpu.available", (name, f"{mem:.1f}")
    except Exception as exc:  # noqa: BLE001
        return "gpu.unavailable", (exc,)
