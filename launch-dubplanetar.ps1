Set-Location $PSScriptRoot

if (-not (Test-Path ".\.venv\Scripts\pythonw.exe")) {
    Write-Host ""
    Write-Host "DubPlanetar — environnement non installé." -ForegroundColor Yellow
    Write-Host "Exécutez d'abord : .\install-dubplanetar.ps1" -ForegroundColor Cyan
    Write-Host ""
    Read-Host "Appuyez sur Entrée pour fermer"
    exit 1
}

& ".\.venv\Scripts\pythonw.exe" -m dub_planetar
