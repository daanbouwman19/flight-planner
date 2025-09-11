# User Installation for Flight Planner

This directory contains a user-specific installation script for Linux that installs Flight Planner to your home directory instead of system-wide.

## Installation

### Quick Install
```bash
# Make the script executable and run it
chmod +x install_user.sh
./install_user.sh
```

### What it does

The `install_user.sh` script:

1. **Installs to user directory** - No root/sudo required
   - Binary: `~/.local/bin/flight_planner`
   - Data: `~/.local/share/flight-planner/`
   - Desktop file: `~/.local/share/applications/`
   - Icons: `~/.local/share/icons/hicolor/`

2. **Supports both source builds and prebuilt binaries**
   - If `Cargo.toml` exists: builds from source using `cargo build --release`
   - If `flight_planner` binary exists: installs the prebuilt binary

3. **KDE Plasma Integration**
   - Updates KDE application cache using `kbuildsycoca5/6`
   - Creates desktop shortcut on Desktop for KDE users
   - Marks desktop file as trusted for KDE
   - Updates desktop database and icon cache

4. **PATH Integration**
   - Checks if `~/.local/bin` is in PATH
   - Provides instructions to add it if missing

## After Installation

### Running the Application
- **From Applications Menu**: Search for "Flight Planner"
- **From Terminal**: `flight_planner` (if `~/.local/bin` is in PATH)
- **Desktop Shortcut**: Double-click the desktop icon (KDE Plasma)

### Adding ~/.local/bin to PATH
If the installer mentions that `~/.local/bin` is not in your PATH, add this line to your `~/.bashrc` or `~/.profile`:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then reload your shell:
```bash
source ~/.bashrc
```

## Uninstallation

To remove Flight Planner:
```bash
# Remove binary
rm ~/.local/bin/flight_planner

# Remove data directory
rm -rf ~/.local/share/flight-planner

# Remove desktop file
rm ~/.local/share/applications/com.github.daan.flight-planner.desktop

# Remove icons
rm -rf ~/.local/share/icons/hicolor/*/apps/com.github.daan.flight-planner.png

# Remove desktop shortcut (KDE)
rm ~/Desktop/com.github.daan.flight-planner.desktop

# Update caches
update-desktop-database ~/.local/share/applications
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor
kbuildsycoca5  # or kbuildsycoca6 for KDE Plasma 6
```

## Requirements

- Linux system
- For source builds: Rust and Cargo
- For KDE Plasma: KDE desktop environment (auto-detected)

## Notes

- The script automatically detects KDE Plasma and provides enhanced integration
- Desktop shortcuts are automatically created and trusted on KDE
- Icons are properly installed for all standard sizes
- The application will look for `airports.db3` in `~/.local/share/flight-planner/` or the current working directory