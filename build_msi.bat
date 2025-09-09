@echo off
setlocal

echo =========================================
echo   Flight Planner WiX Installer Builder
echo =========================================
echo.

:: Check if WiX is installed
where wix.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: WiX Toolset v6+ not found!
    echo.
    echo Please install WiX Toolset v6.0.2+ from:
    echo https://wixtoolset.org/
    echo.
    echo After installation, verify with: wix --version
    echo.
    pause
    exit /b 1
)

:: Build release version
echo [1/3] Building Rust release version...
cargo build --release
if %errorlevel% neq 0 (
    echo ERROR: Cargo build failed
    pause
    exit /b 1
)
echo Rust build complete.
echo.

:: Create output directory
if not exist "dist" mkdir "dist"

:: Compile and build MSI with WiX v6
echo [2/2] Building MSI with WiX v6...
wix build FlightPlanner.wxs -out dist\FlightPlannerSetup.msi
if %errorlevel% neq 0 (
    echo ERROR: WiX build failed
    pause
    exit /b 1
)

echo.
echo =========================================
echo   MSI Installer Created Successfully!
echo =========================================
echo.

if exist "dist\FlightPlannerSetup.msi" (
    for %%f in (dist\FlightPlannerSetup.msi) do set "size=%%~zf"
    set /a "sizeMB=!size!/1048576"
    echo Installer: dist\FlightPlannerSetup.msi
    echo Size: !sizeMB! MB
    echo.
    echo The MSI installer is ready for distribution!
    echo.
    echo Features:
    echo - Professional Windows MSI installer
    echo - Installs to Program Files
    echo - Creates Start Menu shortcuts
    echo - Proper Windows Add/Remove Programs entry
    echo - Supports upgrade/uninstall
    echo - Microsoft-standard installer format
    echo.
    echo To test: Double-click dist\FlightPlannerSetup.msi
    echo.
) else (
    echo ERROR: MSI installer was not created
)

pause
endlocal
