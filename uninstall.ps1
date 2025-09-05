# Flight Planner Windows Uninstallation Script (PowerShell)
# This script removes the Flight Planner application from Windows

param(
    [string]$InstallPath = "$env:ProgramFiles\FlightPlanner",
    [switch]$Help,
    [switch]$Yes
)

# Application details
$AppName = "Flight Planner"
$BinaryName = "flight_planner.exe"

# Function to show help
function Show-Help {
    Write-Host "Flight Planner Windows Uninstallation Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\uninstall.ps1 [OPTIONS]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -InstallPath <path>  Set installation directory (default: `$env:ProgramFiles\FlightPlanner)" -ForegroundColor White
    Write-Host "  -Help               Show this help message" -ForegroundColor White
    Write-Host "  -Yes                Skip confirmation prompt" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\uninstall.ps1                                    # Uninstall from Program Files" -ForegroundColor White
    Write-Host "  .\uninstall.ps1 -InstallPath 'C:\FlightPlanner'   # Uninstall from custom path" -ForegroundColor White
    Write-Host "  .\uninstall.ps1 -Yes                              # Uninstall without confirmation" -ForegroundColor White
    Write-Host ""
}

# Function to check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Function to confirm uninstallation
function Confirm-Uninstall {
    Write-Host "This will remove Flight Planner from your system." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "The following will be removed:" -ForegroundColor White
    Write-Host "  - Installation directory: $InstallPath" -ForegroundColor White
    Write-Host "  - Desktop shortcut" -ForegroundColor White
    Write-Host "  - Start Menu shortcut" -ForegroundColor White
    Write-Host ""
    Write-Host "Your airports.db3 and data.db files will NOT be removed." -ForegroundColor Green
    Write-Host ""
    
    $response = Read-Host "Are you sure you want to continue? [y/N]"
    if ($response -notmatch '^[Yy]$') {
        Write-Host "[INFO] Uninstallation cancelled." -ForegroundColor Blue
        exit 0
    }
}

# Function to remove files
function Remove-Files {
    Write-Host "[INFO] Removing application files..." -ForegroundColor Blue
    
    # Remove installation directory
    if (Test-Path $InstallPath) {
        try {
            Remove-Item $InstallPath -Recurse -Force
            Write-Host "[SUCCESS] Installation directory removed: $InstallPath" -ForegroundColor Green
        }
        catch {
            Write-Host "[ERROR] Failed to remove installation directory: $InstallPath" -ForegroundColor Red
            Write-Host "You may need to manually remove this directory." -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "[WARNING] Installation directory not found: $InstallPath" -ForegroundColor Yellow
    }
}

# Function to remove shortcuts
function Remove-Shortcuts {
    Write-Host "[INFO] Removing shortcuts..." -ForegroundColor Blue
    
    # Remove desktop shortcut
    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $DesktopShortcut = "$DesktopPath\$AppName.lnk"
    
    if (Test-Path $DesktopShortcut) {
        try {
            Remove-Item $DesktopShortcut -Force
            Write-Host "[SUCCESS] Desktop shortcut removed: $DesktopShortcut" -ForegroundColor Green
        }
        catch {
            Write-Host "[WARNING] Failed to remove desktop shortcut: $DesktopShortcut" -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "[INFO] Desktop shortcut not found: $DesktopShortcut" -ForegroundColor Blue
    }
    
    # Remove Start Menu shortcut
    $StartMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs"
    $StartMenuShortcut = "$StartMenuPath\$AppName.lnk"
    
    if (Test-Path $StartMenuShortcut) {
        try {
            Remove-Item $StartMenuShortcut -Force
            Write-Host "[SUCCESS] Start Menu shortcut removed: $StartMenuShortcut" -ForegroundColor Green
        }
        catch {
            Write-Host "[WARNING] Failed to remove Start Menu shortcut: $StartMenuShortcut" -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "[INFO] Start Menu shortcut not found: $StartMenuShortcut" -ForegroundColor Blue
    }
}

# Function to handle user data
function Handle-UserData {
    Write-Host ""
    Write-Host "[INFO] Your airports.db3 and data.db files have been preserved." -ForegroundColor Green
    Write-Host "[INFO] You can safely remove them manually if no longer needed." -ForegroundColor Blue
}

# Main uninstallation process
function Main {
    Write-Host "Flight Planner Windows Uninstallation Script" -ForegroundColor Cyan
    Write-Host "===============================================" -ForegroundColor Cyan
    Write-Host ""
    
    # Check if running as administrator
    if (-not (Test-Administrator)) {
        Write-Host "[ERROR] This script must be run as Administrator" -ForegroundColor Red
        Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
        exit 1
    }
    
    # Confirm uninstallation unless -Yes flag is used
    if (-not $Yes) {
        Confirm-Uninstall
    }
    
    Remove-Files
    Remove-Shortcuts
    Handle-UserData
    
    Write-Host ""
    Write-Host "[SUCCESS] Uninstallation complete!" -ForegroundColor Green
}

# Show help if requested
if ($Help) {
    Show-Help
    exit 0
}

# Run main function
Main
