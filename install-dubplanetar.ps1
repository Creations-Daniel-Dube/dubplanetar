#Requires -Version 5.1
<#
.SYNOPSIS
    Installe DubPlanetar et toutes ses dépendances Python.

.DESCRIPTION
    Crée l'environnement virtuel (.venv), installe les paquets pip,
    installe le projet en mode editable et compile les traductions.

.NOTES
    Prérequis manuels (non installables par ce script) :
    - Windows 10/11 64 bits
    - Python 3.11 ou supérieur (https://python.org)
    - GPU NVIDIA + pilotes CUDA 12.x
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
Set-Location $ProjectRoot

function Write-Step([string]$Message) {
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Find-Python {
    $candidates = @("python", "py")
    foreach ($name in $candidates) {
        try {
            $versionText = & $name -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>$null
            if ($LASTEXITCODE -ne 0 -or -not $versionText) { continue }
            $parts = $versionText.Trim().Split(".")
            $major = [int]$parts[0]
            $minor = [int]$parts[1]
            if ($major -gt 3 -or ($major -eq 3 -and $minor -ge 11)) {
                return @{ Command = $name; Version = $versionText.Trim() }
            }
            Write-Warning "Python $versionText trouvé, mais 3.11+ est requis."
        } catch {
            continue
        }
    }
    return $null
}

Write-Host "DubPlanetar — installation" -ForegroundColor Green
Write-Host "Dossier : $ProjectRoot"

Write-Step "Recherche de Python 3.11+"
$python = Find-Python
if (-not $python) {
    Write-Host ""
    Write-Host "ERREUR : Python 3.11+ introuvable." -ForegroundColor Red
    Write-Host "Installez Python depuis https://www.python.org/downloads/"
    Write-Host "Cochez « Add python.exe to PATH » pendant l'installation."
    exit 1
}
Write-Host "Python $($python.Version) OK ($($python.Command))"

$venvPython = Join-Path $ProjectRoot ".venv\Scripts\python.exe"
$venvPip = Join-Path $ProjectRoot ".venv\Scripts\pip.exe"

Write-Step "Création de l'environnement virtuel (.venv)"
if (-not (Test-Path $venvPython)) {
    & $python.Command -m venv ".venv"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERREUR : impossible de créer .venv" -ForegroundColor Red
        exit 1
    }
    Write-Host "Environnement virtuel créé."
} else {
    Write-Host "Environnement virtuel déjà présent."
}

Write-Step "Mise à jour de pip"
& $venvPython -m pip install --upgrade pip

Write-Step "Installation des dépendances (requirements.txt)"
& $venvPip install -r "requirements.txt"
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERREUR : échec de pip install -r requirements.txt" -ForegroundColor Red
    exit 1
}

Write-Step "Installation de DubPlanetar (mode editable)"
& $venvPip install -e .
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERREUR : échec de pip install -e ." -ForegroundColor Red
    exit 1
}

Write-Step "Compilation des traductions (.ts -> .qm)"
& $venvPython "scripts\compile_translations.py"
if ($LASTEXITCODE -ne 0) {
    Write-Host "AVERTISSEMENT : compilation des traductions échouée." -ForegroundColor Yellow
    Write-Host "L'interface fonctionnera, mais certaines langues peuvent manquer."
}

Write-Step "Vérification CUDA / GPU"
$cudaCheck = & $venvPython -c @"
try:
    import cupy as cp
    dev = cp.cuda.Device(0)
    dev.use()
    props = cp.cuda.runtime.getDeviceProperties(0)
    name = props['name'].decode() if isinstance(props['name'], bytes) else str(props['name'])
    mem = dev.mem_info[1] / (1024 ** 3)
    print(f'OK|{name}|{mem:.1f}')
except Exception as exc:
    print(f'FAIL|{exc}')
"@ 2>&1

if ($cudaCheck -match "^OK\|") {
    $parts = $cudaCheck -split "\|"
    Write-Host "GPU détecté : $($parts[1]) ($($parts[2]) Go VRAM libre)" -ForegroundColor Green
} else {
    $detail = ($cudaCheck -replace "^FAIL\|", "").Trim()
    Write-Host "CUDA / GPU non disponible : $detail" -ForegroundColor Yellow
    Write-Host "DubPlanetar s'ouvrira, mais l'empilement GPU ne fonctionnera pas."
    Write-Host "Vérifiez vos pilotes NVIDIA et CUDA 12.x."
}

Write-Host ""
Write-Host "Installation terminée." -ForegroundColor Green
Write-Host ""
Write-Host "Lancer l'application :" -ForegroundColor White
Write-Host "  .\launch-dubplanetar.ps1"
Write-Host "  ou : .\.venv\Scripts\python.exe -m dub_planetar"
Write-Host ""
