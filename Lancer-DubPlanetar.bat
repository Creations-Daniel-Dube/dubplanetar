@echo off
REM Lance DubPlanetar (double-clic). Prefere le VBS pour eviter la fenetre console.
setlocal EnableExtensions
cd /d "%~dp0"

if exist "%~dp0Lancer-DubPlanetar.vbs" (
  wscript //nologo "%~dp0Lancer-DubPlanetar.vbs"
  exit /b 0
)

set "PY="
where python >nul 2>&1 && set "PY=python"
if not defined PY (
  where py >nul 2>&1 && set "PY=py -3"
)
if not defined PY (
  echo.
  echo ERREUR : Python introuvable.
  echo Double-cliquez d'abord sur Installer-DubPlanetar.bat
  echo.
  pause
  exit /b 1
)

%PY% launch_dubplanetar.py %*
exit /b %ERRORLEVEL%
