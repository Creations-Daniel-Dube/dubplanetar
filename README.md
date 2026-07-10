# DubPlanetar (Nvidia CUDA)

📖 **[Version française](README.fr.md)**

**GPU (Nvidia CUDA) Sun / Moon stacking** for RAW AVI videos captured with a **SeeStar** device (S50, S30, S30 Pro).

DubPlanetar turns a raw video sequence into a final super-resolved 16-bit TIFF image, optimized to reveal detail on the solar or lunar disk.

---

## Table of contents

- [Overview](#overview)
- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Launching](#launching)
- [Usage](#usage)
- [Sun and Moon profiles](#sun-and-moon-profiles)
- [Detailed settings](#detailed-settings)
- [Technical pipeline](#technical-pipeline)
- [Output format](#output-format)
- [Supported languages](#supported-languages)
- [Troubleshooting](#troubleshooting)
- [Credits](#credits)

---



## Overview

Planetary astrophotography with a SeeStar produces AVI videos in RAW format (undebayered sensor data). Each individual frame is noisy and affected by atmospheric turbulence. Stacking consists of:

1. Selecting the sharpest frames
2. Aligning them with sub-pixel precision
3. Combining them to improve the signal-to-noise ratio
4. Applying image processing tailored to the target (Sun or Moon)

DubPlanetar automates this entire pipeline on NVIDIA GPUs via CUDA, with an intuitive graphical interface.

**Result:** a `*_stacked.tiff` file cropped on the disk, super-resolved (×3 by default), ready for post-processing or display.

---



## Features

- **PySide6** (Qt) graphical interface with result preview
- **100 % GPU** processing (CuPy / CUDA) — sharpness scoring, alignment, stacking, debayering
- Preset **Sun** and **Moon** profiles with optimized settings
- Automatic Bayer pattern detection (BGGR, GRBG, GBRG, RGGB)
- Automatic disk cropping with adjustable margin
- Super-resolution via **drizzle** (×3 factor)
- Advanced tonal processing: gamma, solar stretch, center flattening, multi-scale sharpening
- Multilingual interface: **French, English, Spanish, German**
- Automatic per-profile preference saving

---



## Requirements


| Component     | Requirement                                              |
| ------------- | -------------------------------------------------------- |
| OS            | Windows 10 or 11 (64-bit)                                |
| GPU           | NVIDIA with CUDA 12.x support (tested on RTX 3060 12 GB) |
| Drivers       | Recent NVIDIA drivers with CUDA 12                       |
| Python        | 3.11 or later                                            |
| Device        | SeeStar S50, S30, or S30 Pro                             |
| Source format | RAW AVI files (uncompressed, Bayer sensor)               |


> **CUDA note:** the project uses `cupy-cuda12x`. If you have CUDA 11, install `cupy-cuda11x` in `requirements.txt` instead.

---



## Installation



### 1. Clone the repository

```powershell
git clone https://github.com/Creations-Daniel-Dube/dubplanetar.git
cd dubplanetar
```



### 2. Create a Python virtual environment

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
```



### 3. Install dependencies

```powershell
pip install -r requirements.txt
pip install -e .
```



### 4. Compile translations (if `.qm` files are missing)

```powershell
python scripts\compile_translations.py
```

> Compiled `.qm` files are included in the repository; this step is only needed if you modify the `.ts` translation files.



### 5. Verify CUDA

```powershell
python -c "import cupy as cp; d=cp.cuda.Device(0); d.use(); print(cp.cuda.runtime.getDeviceProperties(0)['name'])"
```

If this command prints your GPU name, the installation was successful.

---



## Launching



### Via Python

```powershell
python -m dub_planetar
```



### Via the PowerShell script (no console)

```powershell
.\launch-dubplanetar.ps1
```



### Via the installed command

```powershell
dubplanetar
```

---



## Usage

1. **Launch** DubPlanetar
2. **Choose the target**: ☀ Sun or ☾ Moon (loads the corresponding profile)
3. **Select** a SeeStar RAW AVI file via *Browse…*
4. **Adjust** settings if needed (profile defaults work in most cases)
5. Click **Stack**
6. Follow progress in the bar and preview on the right
7. The `*_stacked.tiff` file is created **next to the source video**

---



## Sun and Moon profiles

Profiles preconfigure settings for each target. Your settings are saved separately per profile.


| Setting        | Sun profile | Moon profile |
| -------------- | ----------- | ------------ |
| Frames kept    | 50 %        | 50 %         |
| White balance  | Disabled    | Enabled      |
| Flatten center | 70 %        | Disabled     |
| Debayer        | Enabled     | Enabled      |
| Auto crop      | Enabled     | Enabled      |
| Drizzle ×3     | Enabled     | Enabled      |


---



## Detailed settings



### Frame selection

- **Frames kept**: percentage of the sharpest frames to retain (10–100 %). Lower = sharper, but less signal.
- **Frame limit**: maximum number of frames to read (0 = all).



### RAW processing

- **Debayer**: converts the Bayer sensor to color. Disable for monochrome stacking.
- **Bayer pattern**: `AUTO` (recommended), or force BGGR / GRBG / GBRG / RGGB.
- **Auto white balance**: corrects the green cast typical of SeeStar sensors.



### Cropping and super-resolution

- **Auto crop**: detects the disk and crops automatically.
- **Disk margin**: extra space around the detected disk (1–30 %).
- **Drizzle ×3**: super-resolved stacking (final resolution ×3 in width and height).



### Tonal processing

- **Flatten center**: reduces overexposure at the center of the Sun (0 = disabled).
- **Gamma**: reveals dark details (> 1.0 brightens shadows).
- **Multi-scale sharpening**: wavelet enhancement (0 = disabled).
- **Sun compression (asinh)**: adaptive stretch for the solar disk (0 = disabled).
- **Sky background → black**: automatic background subtraction.
- **Black point**: low threshold for black (in %).
- **Protect highlights**: prevents burning the center of the disk.

---



## Technical pipeline

Processing runs entirely on the GPU:

```
RAW AVI video
    │
    ▼
1. Frame reading (OpenCV)
    │
    ▼
2. GPU transfer + sharpness score (Laplacian variance)
    │
    ▼
3. Best frame selection
    │
    ▼
4. Sub-pixel alignment (phase correlation, cuFFT)
    │
    ▼
5. Automatic disk cropping (Sun/Moon)
    │
    ▼
6. Dense BGGR debayer (bilinear interpolation)
    │
    ▼
7. Super-resolved stacking (sub-pixel shift-and-add, drizzle ×3)
    │
    ▼
8. White balance + normalization → 16-bit TIFF
```

---



## Output format

- **File**: `<video_name>_stacked.tiff` (same folder as the source)
- **Bit depth**: 16 bits per channel
- **Channels**: RGB (debayer enabled) or monochrome
- **Resolution**: approximately ×3 the cropped disk size (with drizzle enabled)
- **Color space**: linear, normalized with gamma applied

---



## Supported languages

The interface adapts automatically to the system language:


| Code | Language |
| ---- | -------- |
| `fr` | French   |
| `en` | English  |
| `es` | Spanish  |
| `de` | German   |


To modify translations, edit the `.ts` files in `src/dub_planetar/translations/`, then recompile with `python scripts/compile_translations.py`.

---



## Troubleshooting



### "GPU unavailable" at startup

- Verify you have an NVIDIA card with up-to-date drivers
- Install CUDA 12.x or adapt `requirements.txt` to your CUDA version
- Test: `python -c "import cupy; print(cupy.cuda.runtime.getDeviceCount())"`



### CuPy import error

Make sure the CuPy variant matches your CUDA:

```powershell
# CUDA 12.x (default)
pip install cupy-cuda12x[ctk]

# CUDA 11.x
pip install cupy-cuda11x[ctk]
```



### Interface is in English despite a French system

Compiled `.qm` files may be missing. Run:

```powershell
python scripts\compile_translations.py
```



### Video won't open

- Verify the file is an uncompressed SeeStar RAW AVI
- Try limiting the number of frames to test on a short clip



### Result too dark or too bright

- Adjust **gamma** (1.2 by default)
- For the Sun, enable **Flatten center** (70 % in the Sun profile)
- Check **black point** and highlight protection

---



## Credits

**DubPlanetar** — Créations Daniel Dubé

GPU stacking software for SeeStar planetary astrophotography.

---



## License

All rights reserved © Créations Daniel Dubé.