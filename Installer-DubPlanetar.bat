@echo off
REM Installe DubPlanetar (double-clic) — crée le .venv et les dépendances.
setlocal EnableExtensions
cd /d "%~dp0"

set "PY="
where python >nul 2>&1 && set "PY=python"
if not defined PY (
  where py >nul 2>&1 && set "PY=py -3"
)
if not defined PY (
  echo.
  echo ERREUR : Python 3.11+ introuvable.
  echo Installez Python depuis https://www.python.org/downloads/
  echo Cochez « Add python.exe to PATH » pendant l'installation.
  echo.
  pause
  exit /b 1
)

echo DubPlanetar — installation (Windows^)
echo Dossier : %CD%
echo.
%PY% install_dubplanetar.py %*
set "ERR=%ERRORLEVEL%"
echo.
if not "%ERR%"=="0" (
  echo Installation terminee avec des erreurs (code %ERR%^).
) else (
  echo Installation terminee.
)
echo.
pause
exit /b %ERR%
