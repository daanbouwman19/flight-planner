@echo off
REM Flight Planner Windows Uninstallation Script (Batch)
REM This script removes the Flight Planner application from Windows

setlocal enabledelayedexpansion

REM Default values
set "INSTALL_PATH=%ProgramFiles%\FlightPlanner"
set "APP_NAME=Flight Planner"
set "YES_FLAG=0"

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :main
if "%~1"=="--help" goto :show_help
if "%~1"=="-h" goto :show_help
if "%~1"=="--yes" (
    set "YES_FLAG=1"
    shift
    goto :parse_args
)
if "%~1"=="-y" (
    set "YES_FLAG=1"
    shift
    goto :parse_args
)
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
echo Flight Planner Windows Uninstallation Script
echo.
echo Usage: uninstall.bat [OPTIONS]
echo.
echo Options:
echo   --install-path ^<path^>  Set installation directory (default: %%ProgramFiles%%\FlightPlanner)
echo   -p ^<path^>             Set installation directory (default: %%ProgramFiles%%\FlightPlanner)
echo   --yes                   Skip confirmation prompt
echo   -y                      Skip confirmation prompt
echo   --help                  Show this help message
echo   -h                      Show this help message
echo.
echo Examples:
echo   uninstall.bat                                    # Uninstall from Program Files
echo   uninstall.bat --install-path "C:\FlightPlanner"  # Uninstall from custom path
echo   uninstall.bat --yes                              # Uninstall without confirmation
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

:confirm_uninstall
if "%YES_FLAG%"=="1" goto :eof

echo This will remove Flight Planner from your system.
echo.
echo The following will be removed:
echo   - Installation directory: %INSTALL_PATH%
echo   - Desktop shortcut
echo   - Start Menu shortcut
echo.
echo Your airports.db3 and data.db files will NOT be removed.
echo.
set /p "response=Are you sure you want to continue? [y/N]: "
if /i not "%response%"=="y" (
    echo [INFO] Uninstallation cancelled.
    goto :end
)
goto :eof

:remove_files
echo [INFO] Removing application files...

if exist "%INSTALL_PATH%" (
    rmdir /s /q "%INSTALL_PATH%" 2>nul
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to remove installation directory: %INSTALL_PATH%
        echo You may need to manually remove this directory.
    ) else (
        echo [SUCCESS] Installation directory removed: %INSTALL_PATH%
    )
) else (
    echo [WARNING] Installation directory not found: %INSTALL_PATH%
)
goto :eof

:remove_shortcuts
echo [INFO] Removing shortcuts...

REM Remove desktop shortcut
set "DESKTOP_PATH=%USERPROFILE%\Desktop"
set "DESKTOP_SHORTCUT=%DESKTOP_PATH%\%APP_NAME%.lnk"

if exist "%DESKTOP_SHORTCUT%" (
    del "%DESKTOP_SHORTCUT%" 2>nul
    if %errorLevel% neq 0 (
        echo [WARNING] Failed to remove desktop shortcut: %DESKTOP_SHORTCUT%
    ) else (
        echo [SUCCESS] Desktop shortcut removed: %DESKTOP_SHORTCUT%
    )
) else (
    echo [INFO] Desktop shortcut not found: %DESKTOP_SHORTCUT%
)

REM Remove Start Menu shortcut
set "START_MENU_PATH=%ProgramData%\Microsoft\Windows\Start Menu\Programs"
set "START_MENU_SHORTCUT=%START_MENU_PATH%\%APP_NAME%.lnk"

if exist "%START_MENU_SHORTCUT%" (
    del "%START_MENU_SHORTCUT%" 2>nul
    if %errorLevel% neq 0 (
        echo [WARNING] Failed to remove Start Menu shortcut: %START_MENU_SHORTCUT%
    ) else (
        echo [SUCCESS] Start Menu shortcut removed: %START_MENU_SHORTCUT%
    )
) else (
    echo [INFO] Start Menu shortcut not found: %START_MENU_SHORTCUT%
)
goto :eof

:handle_user_data
echo.
echo [INFO] Your airports.db3 and data.db files have been preserved.
echo [INFO] You can safely remove them manually if no longer needed.
goto :eof

:main
echo Flight Planner Windows Uninstallation Script
echo ===============================================
echo.

call :check_admin
call :confirm_uninstall
call :remove_files
call :remove_shortcuts
call :handle_user_data

echo.
echo [SUCCESS] Uninstallation complete!

:end
endlocal
