#Requires -Version 5.1
<#
.SYNOPSIS
    Installe DubPlanetar et toutes ses dépendances Python.

.DESCRIPTION
    Délègue à install_dubplanetar.py qui crée un .venv natif Windows
    (.venv\Scripts\) — local à la machine, exclu de Git.

.NOTES
    Prérequis manuels (non installables par ce script) :
    - Windows 10/11 64 bits
    - Python 3.11 ou supérieur (https://python.org)
    - GPU NVIDIA + pilotes CUDA 12.x
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$installer = Join-Path $PSScriptRoot "install_dubplanetar.py"
if (-not (Test-Path $installer)) {
    Write-Host "ERREUR : install_dubplanetar.py introuvable." -ForegroundColor Red
    exit 1
}

$systemPython = $null
foreach ($name in @("python", "py")) {
    try {
        $versionText = & $name -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>$null
        if ($LASTEXITCODE -ne 0 -or -not $versionText) { continue }
        $parts = $versionText.Trim().Split(".")
        $major = [int]$parts[0]
        $minor = [int]$parts[1]
        if ($major -gt 3 -or ($major -eq 3 -and $minor -ge 11)) {
            $systemPython = $name
            break
        }
    } catch {
        continue
    }
}

if (-not $systemPython) {
    Write-Host "ERREUR : Python 3.11+ introuvable." -ForegroundColor Red
    Write-Host "Installez Python depuis https://www.python.org/downloads/"
    Write-Host "Cochez « Add python.exe to PATH » pendant l'installation."
    exit 1
}

& $systemPython $installer @args
exit $LASTEXITCODE
