#!/usr/bin/env bash

set -euo pipefail

# Flight Planner system installer - installs system-wide (requires root)
# For user installation (recommended), use: ./install.sh

APP_NAME="flight_planner"
APP_ID="com.github.daan.flight-planner"
PREFIX="/usr/local"
ICON_SIZES=(16 22 24 32 48 64 128 256 512)

print_help() {
    echo "Flight Planner System Installer"
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -p, --prefix DIR   Install prefix (default: /usr/local)"
    echo "  -h, --help         Show this help"
    echo ""
    echo "This installer installs Flight Planner system-wide (requires root):"
    echo "  - For user installation (recommended): use ./install.sh"
    echo ""
    echo "Auto-detects installation mode:"
    echo "  - Source build: if Cargo.toml exists, builds from source"
    echo "  - Prebuilt: if binary exists in current dir, installs it"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -p|--prefix)
            if [[ -z "${2-}" ]]; then
                echo "Error: Missing argument for '$1'" >&2
                exit 1
            fi
            PREFIX="$2"
            shift 2
            ;;
        -h|--help) print_help; exit 0;;
        *) echo "Unknown option: $1"; exit 1;;
    esac
done

BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share"
DESKTOPDIR="$DATADIR/applications"
ICONDIR="$DATADIR/icons/hicolor"
SHAREAPPDIR="$DATADIR/flight-planner"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    echo "Error: Do not run as root. This script will use sudo when needed." >&2
    exit 1
fi

echo "Installing Flight Planner to prefix: $PREFIX"

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

# Create directories
sudo install -d "$BINDIR" "$DESKTOPDIR" "$ICONDIR" "$SHAREAPPDIR"

# Install binary
sudo install -m 0755 "$BINARY_PATH" "$BINDIR/$APP_NAME"

# Install desktop file
if [[ -f "./$APP_ID.desktop" ]]; then
    # Copy desktop file and ensure it points to the installed binary location
    sed "s|^Exec=flight_planner|Exec=\"$BINDIR/flight_planner\"|" "./$APP_ID.desktop" > "/tmp/$APP_ID.desktop"
    sudo install -m 0644 "/tmp/$APP_ID.desktop" "$DESKTOPDIR/$APP_ID.desktop"
    rm "/tmp/$APP_ID.desktop"
else
    # Create desktop file if it doesn't exist
    sudo tee "$DESKTOPDIR/$APP_ID.desktop" > /dev/null << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Flight Planner
Comment=Flight planning application
Exec="$BINDIR/flight_planner"
Icon=com.github.daan.flight-planner
Terminal=false
Categories=Utility;Science;
StartupNotify=true
StartupWMClass=com.github.daan.flight-planner
EOF
fi

# Install icons
for s in "${ICON_SIZES[@]}"; do
    src="./assets/icons/icon-${s}x${s}.png"
    if [[ -f "$src" ]]; then
        sudo install -d "$ICONDIR/${s}x${s}/apps"
        sudo install -m 0644 "$src" "$ICONDIR/${s}x${s}/apps/$APP_ID.png"
    fi
done

# Install default aircrafts.csv (optional)
if [[ -f "./aircrafts.csv" ]]; then
    sudo install -m 0644 "./aircrafts.csv" "$SHAREAPPDIR/aircrafts.csv"
fi

# Refresh caches - standard XDG desktop integration
if command -v update-desktop-database >/dev/null 2>&1; then
    sudo update-desktop-database "$DESKTOPDIR" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    sudo gtk-update-icon-cache -f -t "$ICONDIR" || true
fi

echo ""
echo "✓ Installation complete!"
echo "Launch from applications menu (all desktop environments) or run: $APP_NAME"
echo ""
echo "Note: Provide airports.db3 at ~/.local/share/flight-planner/airports.db3"
echo "      or run from a directory containing airports.db3"
