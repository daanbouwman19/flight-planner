# Flight Planner Windows Installation Script (PowerShell)
# This script installs the Flight Planner application on Windows

param(
    [string]$InstallPath = "$env:ProgramFiles\FlightPlanner",
    [switch]$Help
)

# Application details
$AppName = "Flight Planner"
$AppId = "com.github.daan.flight-planner"
$BinaryName = "flight_planner.exe"
$Version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value

# Function to show help
function Show-Help {
    Write-Host "Flight Planner Windows Installation Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\install.ps1 [OPTIONS]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -InstallPath <path>  Set installation directory (default: `$env:ProgramFiles\FlightPlanner)" -ForegroundColor White
    Write-Host "  -Help               Show this help message" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\install.ps1                                    # Install to Program Files" -ForegroundColor White
    Write-Host "  .\install.ps1 -InstallPath 'C:\FlightPlanner'   # Install to custom path" -ForegroundColor White
    Write-Host ""
}

# Function to check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Function to check dependencies
function Test-Dependencies {
    Write-Host "[INFO] Checking dependencies..." -ForegroundColor Blue
    
    # Check if cargo is available
    try {
        $null = Get-Command cargo -ErrorAction Stop
        Write-Host "[SUCCESS] Rust/Cargo found" -ForegroundColor Green
    }
    catch {
        Write-Host "[ERROR] Rust/Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
        exit 1
    }
    
    # Check if running as administrator
    if (-not (Test-Administrator)) {
        Write-Host "[ERROR] This script must be run as Administrator" -ForegroundColor Red
        Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
        exit 1
    }
}

# Function to build the application
function Build-Application {
    Write-Host "[INFO] Building Flight Planner..." -ForegroundColor Blue
    
    try {
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "Build failed"
        }
        Write-Host "[SUCCESS] Build completed" -ForegroundColor Green
    }
    catch {
        Write-Host "[ERROR] Build failed!" -ForegroundColor Red
        exit 1
    }
}

# Function to create installation directory
function New-InstallationDirectory {
    Write-Host "[INFO] Creating installation directory..." -ForegroundColor Blue
    
    if (-not (Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
        Write-Host "[SUCCESS] Directory created: $InstallPath" -ForegroundColor Green
    }
    else {
        Write-Host "[INFO] Directory already exists: $InstallPath" -ForegroundColor Yellow
    }
}

# Function to install files
function Install-Files {
    Write-Host "[INFO] Installing application files..." -ForegroundColor Blue
    
    # Install binary
    $BinarySource = "target\release\$BinaryName"
    $BinaryDest = "$InstallPath\$BinaryName"
    
    if (Test-Path $BinarySource) {
        Copy-Item $BinarySource $BinaryDest -Force
        Write-Host "[SUCCESS] Binary installed: $BinaryDest" -ForegroundColor Green
    }
    else {
        Write-Host "[ERROR] Binary not found: $BinarySource" -ForegroundColor Red
        exit 1
    }
    
    # Install icon
    $IconSource = "assets\icons\icon-64x64.png"
    $IconDest = "$InstallPath\icon.png"
    
    if (Test-Path $IconSource) {
        Copy-Item $IconSource $IconDest -Force
        Write-Host "[SUCCESS] Icon installed: $IconDest" -ForegroundColor Green
    }
    else {
        Write-Host "[WARNING] Icon not found: $IconSource" -ForegroundColor Yellow
    }
}

# Function to create desktop shortcut
function New-DesktopShortcut {
    Write-Host "[INFO] Creating desktop shortcut..." -ForegroundColor Blue
    
    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $ShortcutPath = "$DesktopPath\$AppName.lnk"
    
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = "$InstallPath\$BinaryName"
    $Shortcut.WorkingDirectory = $InstallPath
    $Shortcut.Description = "Flight planning application"
    $Shortcut.IconLocation = "$InstallPath\icon.png"
    $Shortcut.Save()
    
    Write-Host "[SUCCESS] Desktop shortcut created: $ShortcutPath" -ForegroundColor Green
}

# Function to create start menu shortcut
function New-StartMenuShortcut {
    Write-Host "[INFO] Creating Start Menu shortcut..." -ForegroundColor Blue
    
    $StartMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs"
    $ShortcutPath = "$StartMenuPath\$AppName.lnk"
    
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = "$InstallPath\$BinaryName"
    $Shortcut.WorkingDirectory = $InstallPath
    $Shortcut.Description = "Flight planning application"
    $Shortcut.IconLocation = "$InstallPath\icon.png"
    $Shortcut.Save()
    
    Write-Host "[SUCCESS] Start Menu shortcut created: $ShortcutPath" -ForegroundColor Green
}

# Function to show post-installation instructions
function Show-PostInstallInstructions {
    $AppDataPath = "$env:APPDATA\FlightPlanner"
    
    Write-Host ""
    Write-Host "[SUCCESS] Installation complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "[WARNING] IMPORTANT: You need to provide your own airports database!" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "The Flight Planner requires an airports database file (airports.db3) to function." -ForegroundColor White
    Write-Host ""
    Write-Host "Application data directory: $AppDataPath" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Option 1: Place airports.db3 in the application data directory (recommended)" -ForegroundColor Cyan
    Write-Host "  Copy your airports.db3 to: $AppDataPath\airports.db3" -ForegroundColor White
    Write-Host ""
    Write-Host "Option 2: Run the application from the directory containing airports.db3" -ForegroundColor Cyan
    Write-Host "  Navigate to the directory with airports.db3 and run: $InstallPath\$BinaryName" -ForegroundColor White
    Write-Host ""
    Write-Host "The application will automatically create its own data.db file for" -ForegroundColor White
    Write-Host "aircraft and flight history in: $AppDataPath" -ForegroundColor White
    Write-Host ""
    Write-Host "Logs are stored in: $AppDataPath\logs\" -ForegroundColor White
    Write-Host ""
    Write-Host "You can now launch Flight Planner from:" -ForegroundColor Green
    Write-Host "  - Desktop shortcut" -ForegroundColor White
    Write-Host "  - Start Menu" -ForegroundColor White
    Write-Host "  - Command line: $InstallPath\$BinaryName" -ForegroundColor White
}

# Main installation process
function Main {
    Write-Host "Flight Planner Windows Installation Script v$Version" -ForegroundColor Cyan
    Write-Host "=====================================================" -ForegroundColor Cyan
    Write-Host ""
    
    Test-Dependencies
    Build-Application
    New-InstallationDirectory
    Install-Files
    New-DesktopShortcut
    New-StartMenuShortcut
    Show-PostInstallInstructions
}

# Show help if requested
if ($Help) {
    Show-Help
    exit 0
}

# Run main function
Main
