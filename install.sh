#!/usr/bin/env bash

set -euo pipefail

# Flight Planner installer - installs to user directory (default installation method)

APP_NAME="flight_planner"
APP_ID="com.github.daan.flight-planner"
ICON_SIZES=(16 22 24 32 48 64 128 256 512)

# User-specific directories (default installation)
USER_HOME="$HOME"
USER_BIN="$USER_HOME/.local/bin"
USER_DATADIR="$USER_HOME/.local/share"
USER_DESKTOPDIR="$USER_DATADIR/applications"
USER_ICONDIR="$USER_DATADIR/icons/hicolor"
USER_SHAREAPPDIR="$USER_DATADIR/flight-planner"

print_help() {
    echo "Flight Planner Installer"
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help         Show this help"
    echo ""
    echo "This installer installs Flight Planner to your user directory (no root required):"
    echo "  - Binary: ~/.local/bin/"
    echo "  - Data: ~/.local/share/flight-planner/"
    echo "  - Desktop file: ~/.local/share/applications/"
    echo "  - Icons: ~/.local/share/icons/hicolor/"
    echo ""
    echo "Auto-detects installation mode:"
    echo "  - Source build: if Cargo.toml exists, builds from source"
    echo "  - Prebuilt: if binary exists in current dir, installs it"
    echo ""
    echo "Works with all desktop environments (KDE, GNOME, XFCE, etc.)"
    echo ""
    echo "For system-wide installation, use: ./install_system.sh"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) print_help; exit 0;;
        -*)
            echo "Unknown option: $1" >&2
            print_help
            exit 1
            ;;
        *)
            echo "Unknown argument: $1" >&2
            print_help
            exit 1
            ;;
    esac
done

echo "Installing Flight Planner to your user directory: $USER_HOME/.local/"

# Check if running as root (should not for user installation)
if [[ $EUID -eq 0 ]]; then
    echo "Error: Do not run as root. This script installs to your user directory." >&2
    echo "For system-wide installation, use: ./install_system.sh" >&2
    exit 1
fi

# Auto-detect installation mode
if [[ -f "Cargo.toml" ]]; then
    echo "Source build detected (Cargo.toml found) - building from source..."
    
    # Check dependencies
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Error: cargo not found. Install Rust from https://rustup.rs/" >&2
        exit 1
    fi
    
    # Build from source
    echo "Building..."
    cargo build --release
    BINARY_PATH="target/release/$APP_NAME"
elif [[ -f "./$APP_NAME" ]]; then
    echo "Prebuilt binary detected - installing prebuilt binary..."
    BINARY_PATH="./$APP_NAME"
else
    echo "Error: Neither Cargo.toml nor ./$APP_NAME found." >&2
    echo "Run from source directory or extracted release package." >&2
    exit 1
fi

# Create user directories
mkdir -p "$USER_BIN" "$USER_DESKTOPDIR" "$USER_ICONDIR" "$USER_SHAREAPPDIR"

# Install binary
echo "Installing binary to $USER_BIN/$APP_NAME"
install -m 0755 "$BINARY_PATH" "$USER_BIN/$APP_NAME"

# Install desktop file with user-specific paths
echo "Installing desktop file..."
if [[ -f "./$APP_ID.desktop" ]]; then
    # Copy desktop file and ensure it points to the user binary
    sed "s|^Exec=flight_planner|Exec=\"$USER_BIN/flight_planner\"|" "./$APP_ID.desktop" > "$USER_DESKTOPDIR/$APP_ID.desktop"
    chmod 644 "$USER_DESKTOPDIR/$APP_ID.desktop"
else
    # Create desktop file if it doesn't exist
    cat > "$USER_DESKTOPDIR/$APP_ID.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Flight Planner
Comment=Flight planning application
Exec="$USER_BIN/flight_planner"
Icon=com.github.daan.flight-planner
Terminal=false
Categories=Utility;Science;
StartupNotify=true
StartupWMClass=com.github.daan.flight-planner
EOF
fi

# Install icons
echo "Installing icons..."
for s in "${ICON_SIZES[@]}"; do
    src="./assets/icons/icon-${s}x${s}.png"
    if [[ -f "$src" ]]; then
        mkdir -p "$USER_ICONDIR/${s}x${s}/apps"
        install -m 0644 "$src" "$USER_ICONDIR/${s}x${s}/apps/$APP_ID.png"
    fi
done

# Install default aircrafts.csv (optional)
if [[ -f "./aircrafts.csv" ]]; then
    echo "Installing default aircrafts.csv..."
    install -m 0644 "./aircrafts.csv" "$USER_SHAREAPPDIR/aircrafts.csv"
fi

# Install airports.db3 if it exists
if [[ -f "./airports.db3" ]]; then
    echo "Installing airports.db3..."
    install -m 0644 "./airports.db3" "$USER_SHAREAPPDIR/airports.db3"
fi

# Add ~/.local/bin to PATH if not already there
if [[ ":$PATH:" != *":$USER_BIN:"* ]]; then
    echo ""
    echo "Note: $USER_BIN is not in your PATH."
    echo "Add this line to your ~/.bashrc or ~/.profile:"
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Or run this command to add it temporarily:"
    echo "export PATH=\"$USER_BIN:\$PATH\""
fi

# Refresh desktop database and icon cache (user-specific)
echo "Refreshing desktop and icon caches..."

# Update desktop database (user-specific)
if command -v update-desktop-database >/dev/null 2>&1; then
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$USER_DESKTOPDIR" 2>/dev/null || true
fi
fi

# Update icon cache (user-specific)
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$USER_ICONDIR" 2>/dev/null || true
fi

# Standard XDG desktop integration - works across all desktop environments
# The desktop file and icons are now properly installed according to XDG standards
# This ensures the application appears in all desktop environment menus (KDE, GNOME, etc.)

echo ""
echo "✓ Installation complete!"
echo ""
echo "Flight Planner has been installed to your user directory:"
echo "  Binary: $USER_BIN/$APP_NAME"
echo "  Data: $USER_SHAREAPPDIR/"
echo "  Desktop file: $USER_DESKTOPDIR/$APP_ID.desktop"
echo "  Icons: $USER_ICONDIR/"
echo ""
echo "You can now:"
echo "  - Launch from applications menu (all desktop environments)"
echo "  - Run from terminal: flight_planner (if ~/.local/bin is in PATH)"
echo ""
if [[ -f "$USER_SHAREAPPDIR/airports.db3" ]]; then
    echo "✓ airports.db3 has been installed to $USER_SHAREAPPDIR/"
else
    echo "Note: Place airports.db3 at $USER_SHAREAPPDIR/airports.db3"
    echo "      or run from a directory containing airports.db3"
fi
echo ""