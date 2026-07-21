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

echo ========================================================
echo   DubPlanetar — installation (Windows)
echo ========================================================
echo Dossier : %CD%
echo Statut  : en cours...
echo.
%PY% -u install_dubplanetar.py %*
set "ERR=%ERRORLEVEL%"
echo.
if not "%ERR%"=="0" (
  echo ========================================================
  echo   ECHEC DE L'INSTALLATION (code %ERR%^)
  echo ========================================================
) else (
  echo ========================================================
  echo   INSTALLATION TERMINEE
  echo   - Raccourci Bureau : DubPlanetar
  echo   - L'application devrait s'etre ouverte
  echo ========================================================
)
echo.
pause
exit /b %ERR%
