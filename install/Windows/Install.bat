@echo off
REM DubPlanetar — décompresse le pack puis lance l'installation (Windows).
setlocal EnableExtensions
cd /d "%~dp0"

set "ARCHIVE=DubPlanetar-0.6.75.zip"
set "DEST=DubPlanetar"

echo ========================================================
echo   DubPlanetar — installation (Windows)
echo ========================================================
echo.

if not exist "%ARCHIVE%" (
  echo ERREUR : archive introuvable : %ARCHIVE%
  echo Placez Install.bat et %ARCHIVE% dans le meme dossier.
  echo.
  pause
  exit /b 1
)

if exist "%DEST%\" (
  echo [1/3] Le dossier %DEST% existe deja — suppression...
  rmdir /s /q "%DEST%"
  if exist "%DEST%\" (
    echo ERREUR : impossible de supprimer %DEST%
    pause
    exit /b 1
  )
) else (
  echo [1/3] Pret a extraire.
)

echo [2/3] Extraction de %ARCHIVE% ...
where tar >nul 2>&1
if %ERRORLEVEL%==0 (
  tar -xf "%ARCHIVE%"
) else (
  powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -LiteralPath '%ARCHIVE%' -DestinationPath '.' -Force"
)
if errorlevel 1 (
  echo ERREUR : echec de la decompression.
  pause
  exit /b 1
)
echo       Extraction terminee.

if not exist "%DEST%\Installer-DubPlanetar.bat" (
  echo ERREUR : %DEST%\Installer-DubPlanetar.bat introuvable apres extraction.
  pause
  exit /b 1
)

echo [3/3] Installation des dependances (venv, pip, raccourci Bureau)...
echo       Suivez la progression ci-dessous...
echo.

set "PY="
where python >nul 2>&1 && set "PY=python"
if not defined PY (
  where py >nul 2>&1 && set "PY=py -3"
)
if not defined PY (
  echo ERREUR : Python 3.11+ introuvable.
  echo Installez Python depuis https://www.python.org/downloads/
  echo Cochez « Add python.exe to PATH » pendant l'installation.
  echo.
  pause
  exit /b 1
)

pushd "%DEST%"
%PY% -u install_dubplanetar.py
set "ERR=%ERRORLEVEL%"
popd

echo.
if not "%ERR%"=="0" (
  echo ========================================================
  echo   ECHEC DE L'INSTALLATION (code %ERR%)
  echo ========================================================
) else (
  echo ========================================================
  echo   INSTALLATION TERMINEE
  echo   - Raccourci : Bureau \ DubPlanetar
  echo   - L'application devrait s'etre ouverte
  echo ========================================================
)
echo.
pause
exit /b %ERR%
