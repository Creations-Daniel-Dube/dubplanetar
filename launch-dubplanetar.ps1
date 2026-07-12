#Requires -Version 5.1
<#
.SYNOPSIS
    Lance DubPlanetar via le Python du .venv Windows (pythonw si disponible).
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$launcher = Join-Path $PSScriptRoot "launch_dubplanetar.py"
if (-not (Test-Path $launcher)) {
    Write-Host "ERREUR : launch_dubplanetar.py introuvable." -ForegroundColor Red
    exit 1
}

$systemPython = $null
foreach ($name in @("python", "py")) {
    try {
        $null = & $name -c "import sys" 2>$null
        if ($LASTEXITCODE -eq 0) {
            $systemPython = $name
            break
        }
    } catch {
        continue
    }
}

if (-not $systemPython) {
    Write-Host "ERREUR : Python introuvable pour démarrer le lanceur." -ForegroundColor Red
    Write-Host "Exécutez d'abord : .\install-dubplanetar.ps1"
    exit 1
}

& $systemPython $launcher @args
exit $LASTEXITCODE
