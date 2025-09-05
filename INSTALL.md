# Flight Planner Installation Guide

This guide explains how to install the Flight Planner application on your system.

## Prerequisites

Before installing Flight Planner, ensure you have the following:

- **Rust toolchain** (cargo, rustc) - Install from [rustup.rs](https://rustup.rs/)
- **sudo access** - Required for system-wide installation
- **Airports database** - You must provide your own `airports.db3` file

## Installation Methods

### Linux/Unix Systems

#### Method 1: Using the Installation Script (Recommended)

The easiest way to install Flight Planner is using the provided installation script:

```bash
# Clone or download the repository
git clone <repository-url>
cd flight-planner

# Run the installation script
./install.sh
```

The script will:
- Build the application in release mode
- Install the binary to `/usr/local/bin/`
- Install the desktop file to `/usr/local/share/applications/`
- Install icons to `/usr/local/share/icons/hicolor/`
- Update system databases

#### Method 2: Using Make

If you prefer using Make:

```bash
# Build and install
make install

# Or just build
make build
```

#### Method 3: Manual Installation

For custom installation paths or manual control:

```bash
# Build the application
cargo build --release

# Create directories
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/share/applications
for size in 16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512; do
    sudo mkdir -p "/usr/local/share/icons/hicolor/$size/apps"
done

# Install files
sudo cp target/release/flight_planner /usr/local/bin/
sudo cp com.github.daan.flight-planner.desktop /usr/local/share/applications/
for size in 16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512; do
    sudo cp icon.png "/usr/local/share/icons/hicolor/$size/apps/com.github.daan.flight-planner.png"
done

# Update system databases
sudo update-desktop-database /usr/local/share/applications/
sudo gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor/
```

### Windows Systems

#### Method 1: Using PowerShell Script (Recommended)

The easiest way to install Flight Planner on Windows is using the PowerShell script:

```powershell
# Clone or download the repository
git clone <repository-url>
cd flight-planner

# Run PowerShell as Administrator, then execute:
.\install.ps1
```

The script will:
- Build the application in release mode
- Install the binary to `%ProgramFiles%\FlightPlanner\`
- Create desktop and Start Menu shortcuts
- Set up proper file associations

#### Method 2: Using Batch Script

For users who prefer cmd.exe:

```cmd
REM Clone or download the repository
git clone <repository-url>
cd flight-planner

REM Run Command Prompt as Administrator, then execute:
install.bat
```

#### Method 3: Manual Installation

For custom installation paths or manual control:

```cmd
REM Build the application
cargo build --release

REM Create installation directory
mkdir "C:\FlightPlanner"

REM Copy files
copy "target\release\flight_planner.exe" "C:\FlightPlanner\"
copy "icon.png" "C:\FlightPlanner\"

REM Create shortcuts manually or use the application
```

## Custom Installation Prefix

You can install to a different prefix (e.g., `/usr` instead of `/usr/local`):

```bash
# Using the installation script
./install.sh --prefix /usr

# Using Make
make install PREFIX=/usr
```

## Application Data Directory

Flight Planner uses a dedicated application data directory in your home folder to store:
- **Logs**: Application logs and error messages
- **Database**: Aircraft and flight history data (`data.db`)
- **Airports Database**: User-provided airports data (`airports.db3`)

### Directory Locations:
- **Linux/macOS**: `~/.local/share/flight-planner/`
- **Windows**: `%APPDATA%\FlightPlanner\`

## Providing the Airports Database

**IMPORTANT**: Flight Planner requires an airports database file (`airports.db3`) to function. This file is not included with the application and must be provided by the user.

### Option 1: Place in Application Data Directory (Recommended)

**Linux/macOS:**
```bash
cp /path/to/your/airports.db3 ~/.local/share/flight-planner/airports.db3
```

**Windows:**
```cmd
copy "C:\path\to\your\airports.db3" "%APPDATA%\FlightPlanner\airports.db3"
```

### Option 2: Run from Database Directory

```bash
cd /path/to/directory/containing/airports.db3
flight_planner
```

The application will automatically create its own `data.db` file for aircraft and flight history in the application data directory.

## Post-Installation

After installation:

1. **Verify installation**: The application should appear in your application menu
2. **Provide airports database**: Follow the instructions above
3. **Launch**: Start Flight Planner from the application menu or command line

## Uninstallation

### Linux/Unix Systems

#### Using the Uninstall Script

```bash
./uninstall.sh
```

#### Using Make

```bash
make uninstall
```

#### Manual Uninstallation

```bash
sudo rm -f /usr/local/bin/flight_planner
sudo rm -f /usr/local/share/applications/com.github.daan.flight-planner.desktop
for size in 16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512; do
    sudo rm -f "/usr/local/share/icons/hicolor/$size/apps/com.github.daan.flight-planner.png"
done
sudo update-desktop-database /usr/local/share/applications/
sudo gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor/
```

### Windows Systems

#### Using PowerShell Script

```powershell
# Run PowerShell as Administrator, then execute:
.\uninstall.ps1
```

#### Using Batch Script

```cmd
REM Run Command Prompt as Administrator, then execute:
uninstall.bat
```

#### Manual Uninstallation

```cmd
REM Remove installation directory
rmdir /s /q "%ProgramFiles%\FlightPlanner"

REM Remove shortcuts
del "%USERPROFILE%\Desktop\Flight Planner.lnk"
del "%ProgramData%\Microsoft\Windows\Start Menu\Programs\Flight Planner.lnk"
```

## Troubleshooting

### Application Won't Start

1. **Check airports database**: Ensure `airports.db3` is accessible
2. **Check permissions**: Ensure the binary is executable
3. **Check logs**: Look at `~/.local/share/flight-planner/logs/output.log` (Linux/macOS) or `%APPDATA%\FlightPlanner\logs\output.log` (Windows)

### Icon Not Showing

1. **Update icon cache**: `sudo gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor/`
2. **Check desktop file**: Verify the Icon field matches the installed icon name
3. **Restart desktop environment**: Log out and back in

### Desktop Entry Not Appearing

1. **Update desktop database**: `sudo update-desktop-database /usr/local/share/applications/`
2. **Check file permissions**: Ensure the desktop file is readable
3. **Restart desktop environment**: Log out and back in

## Development Installation

For development purposes, you can run the application directly without installation:

```bash
# Run in development mode
cargo run

# Run with CLI mode
cargo run -- --cli
```

## File Locations

After installation, files are located at:

### Linux/macOS:
- **Binary**: `/usr/local/bin/flight_planner`
- **Desktop file**: `/usr/local/share/applications/com.github.daan.flight-planner.desktop`
- **Icons**: `/usr/local/share/icons/hicolor/*/apps/com.github.daan.flight-planner.png`
- **Application data**: `~/.local/share/flight-planner/`
  - **Logs**: `~/.local/share/flight-planner/logs/output.log`
  - **User data**: `~/.local/share/flight-planner/data.db`
  - **Airports database**: `~/.local/share/flight-planner/airports.db3` (user-provided)

### Windows:
- **Binary**: `%ProgramFiles%\FlightPlanner\flight_planner.exe`
- **Application data**: `%APPDATA%\FlightPlanner\`
  - **Logs**: `%APPDATA%\FlightPlanner\logs\output.log`
  - **User data**: `%APPDATA%\FlightPlanner\data.db`
  - **Airports database**: `%APPDATA%\FlightPlanner\airports.db3` (user-provided)

## Support

If you encounter issues during installation:

1. Check the prerequisites
2. Ensure you have the required airports database
3. Check the log file for error messages
4. Verify file permissions and ownership

For additional help, please refer to the project documentation or create an issue in the project repository.
