@echo off
REM Flight Planner Windows Installation Script (Batch)
REM This script installs the Flight Planner application on Windows

setlocal enabledelayedexpansion

REM Default values
set "INSTALL_PATH=%ProgramFiles%\FlightPlanner"
set "APP_NAME=Flight Planner"
set "BINARY_NAME=flight_planner.exe"
set "VERSION=0.1.0"

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :main
if "%~1"=="--help" goto :show_help
if "%~1"=="-h" goto :show_help
if "%~1"=="--install-path" (
    set "INSTALL_PATH=%~2"
    shift
    shift
    goto :parse_args
)
if "%~1"=="-p" (
    set "INSTALL_PATH=%~2"
    shift
    shift
    goto :parse_args
)
shift
goto :parse_args

:show_help
echo Flight Planner Windows Installation Script
echo.
echo Usage: install.bat [OPTIONS]
echo.
echo Options:
echo   --install-path ^<path^>  Set installation directory (default: %%ProgramFiles%%\FlightPlanner)
echo   -p ^<path^>             Set installation directory (default: %%ProgramFiles%%\FlightPlanner)
echo   --help                  Show this help message
echo   -h                      Show this help message
echo.
echo Examples:
echo   install.bat                                    # Install to Program Files
echo   install.bat --install-path "C:\FlightPlanner"  # Install to custom path
echo.
goto :end

:check_admin
REM Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This script must be run as Administrator
    echo Right-click Command Prompt and select "Run as administrator"
    exit /b 1
)
goto :eof

:check_dependencies
echo [INFO] Checking dependencies...

REM Check if cargo is available
cargo --version >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Rust/Cargo not found. Please install Rust from https://rustup.rs/
    exit /b 1
)

echo [SUCCESS] Rust/Cargo found
goto :eof

:build_app
echo [INFO] Building Flight Planner...

cargo build --release
if %errorLevel% neq 0 (
    echo [ERROR] Build failed!
    exit /b 1
)

echo [SUCCESS] Build completed
goto :eof

:create_directory
echo [INFO] Creating installation directory...

if not exist "%INSTALL_PATH%" (
    mkdir "%INSTALL_PATH%" 2>nul
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to create directory: %INSTALL_PATH%
        exit /b 1
    )
    echo [SUCCESS] Directory created: %INSTALL_PATH%
) else (
    echo [INFO] Directory already exists: %INSTALL_PATH%
)
goto :eof

:install_files
echo [INFO] Installing application files...

REM Install binary
if exist "target\release\%BINARY_NAME%" (
    copy "target\release\%BINARY_NAME%" "%INSTALL_PATH%\%BINARY_NAME%" >nul
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to install binary
        exit /b 1
    )
    echo [SUCCESS] Binary installed: %INSTALL_PATH%\%BINARY_NAME%
) else (
    echo [ERROR] Binary not found: target\release\%BINARY_NAME%
    exit /b 1
)

REM Install icon
if exist "icon.png" (
    copy "icon.png" "%INSTALL_PATH%\icon.png" >nul
    if %errorLevel% neq 0 (
        echo [WARNING] Failed to install icon
    ) else (
        echo [SUCCESS] Icon installed: %INSTALL_PATH%\icon.png
    )
) else (
    echo [WARNING] Icon not found: icon.png
)
goto :eof

:create_shortcuts
echo [INFO] Creating shortcuts...

REM Create desktop shortcut
set "DESKTOP_PATH=%USERPROFILE%\Desktop"
set "DESKTOP_SHORTCUT=%DESKTOP_PATH%\%APP_NAME%.lnk"

powershell -Command "& {$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%DESKTOP_SHORTCUT%'); $Shortcut.TargetPath = '%INSTALL_PATH%\%BINARY_NAME%'; $Shortcut.WorkingDirectory = '%INSTALL_PATH%'; $Shortcut.Description = 'Flight planning application'; $Shortcut.IconLocation = '%INSTALL_PATH%\icon.png'; $Shortcut.Save()}" 2>nul

if exist "%DESKTOP_SHORTCUT%" (
    echo [SUCCESS] Desktop shortcut created: %DESKTOP_SHORTCUT%
) else (
    echo [WARNING] Failed to create desktop shortcut
)

REM Create Start Menu shortcut
set "START_MENU_PATH=%ProgramData%\Microsoft\Windows\Start Menu\Programs"
set "START_MENU_SHORTCUT=%START_MENU_PATH%\%APP_NAME%.lnk"

powershell -Command "& {$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%START_MENU_SHORTCUT%'); $Shortcut.TargetPath = '%INSTALL_PATH%\%BINARY_NAME%'; $Shortcut.WorkingDirectory = '%INSTALL_PATH%'; $Shortcut.Description = 'Flight planning application'; $Shortcut.IconLocation = '%INSTALL_PATH%\icon.png'; $Shortcut.Save()}" 2>nul

if exist "%START_MENU_SHORTCUT%" (
    echo [SUCCESS] Start Menu shortcut created: %START_MENU_SHORTCUT%
) else (
    echo [WARNING] Failed to create Start Menu shortcut
)
goto :eof

:show_instructions
set "APPDATA_PATH=%APPDATA%\FlightPlanner"

echo.
echo [SUCCESS] Installation complete!
echo.
echo [WARNING] IMPORTANT: You need to provide your own airports database!
echo.
echo The Flight Planner requires an airports database file (airports.db3) to function.
echo.
echo Application data directory: %APPDATA_PATH%
echo.
echo Option 1: Place airports.db3 in the application data directory (recommended)
echo   Copy your airports.db3 to: %APPDATA_PATH%\airports.db3
echo.
echo Option 2: Run the application from the directory containing airports.db3
echo   Navigate to the directory with airports.db3 and run: %INSTALL_PATH%\%BINARY_NAME%
echo.
echo The application will automatically create its own data.db file for
echo aircraft and flight history in: %APPDATA_PATH%
echo.
echo Logs are stored in: %APPDATA_PATH%\logs\
echo.
echo You can now launch Flight Planner from:
echo   - Desktop shortcut
echo   - Start Menu
echo   - Command line: %INSTALL_PATH%\%BINARY_NAME%
goto :eof

:main
echo Flight Planner Windows Installation Script v%VERSION%
echo =====================================================
echo.

call :check_admin
call :check_dependencies
call :build_app
call :create_directory
call :install_files
call :create_shortcuts
call :show_instructions

:end
endlocal
