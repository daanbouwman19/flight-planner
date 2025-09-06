#!/bin/bash

# Flight Planner Uninstallation Script
# This script removes the Flight Planner application from the system

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Application details
APP_NAME="flight_planner"
APP_ID="com.github.daan.flight-planner"

# Default installation directories
PREFIX="/usr/local"
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share"
DESKTOPDIR="$DATADIR/applications"
ICONDIR="$DATADIR/icons/hicolor"

# Icon sizes
ICON_SIZES=(16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512)

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_error "This script should not be run as root. It will use sudo when needed."
        exit 1
    fi
}

# Function to confirm uninstallation
confirm_uninstall() {
    echo "This will remove Flight Planner from your system."
    echo ""
    echo "The following will be removed:"
    echo "  - Binary: $BINDIR/$APP_NAME"
    echo "  - Desktop file: $DESKTOPDIR/$APP_ID.desktop"
    echo "  - Icons: $ICONDIR/*/apps/$APP_ID.png"
    echo ""
    echo "Your application data (~/.local/share/flight-planner/) will NOT be removed."
    echo ""
    read -p "Are you sure you want to continue? [y/N]: " -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled."
        exit 0
    fi
}

# Function to remove files
remove_files() {
    print_info "Removing application files..."
    
    # Remove binary
    if [[ -f "$BINDIR/$APP_NAME" ]]; then
        sudo rm -f "$BINDIR/$APP_NAME"
        print_success "Binary removed"
    else
        print_warning "Binary not found at $BINDIR/$APP_NAME"
    fi
    
    # Remove desktop file
    if [[ -f "$DESKTOPDIR/$APP_ID.desktop" ]]; then
        sudo rm -f "$DESKTOPDIR/$APP_ID.desktop"
        print_success "Desktop file removed"
    else
        print_warning "Desktop file not found at $DESKTOPDIR/$APP_ID.desktop"
    fi
    
    # Remove icons
    local icons_removed=0
    for size in "${ICON_SIZES[@]}"; do
        if [[ -f "$ICONDIR/$size/apps/$APP_ID.png" ]]; then
            sudo rm -f "$ICONDIR/$size/apps/$APP_ID.png"
            ((icons_removed++))
        fi
    done
    
    if [[ $icons_removed -gt 0 ]]; then
        print_success "$icons_removed icon files removed"
    else
        print_warning "No icon files found"
    fi
}

# Function to handle user data directory
handle_user_data_directory() {
    local user_data_dir="$HOME/.local/share/flight-planner"
    if [[ -d "$user_data_dir" ]]; then
        echo ""
        print_info "Application data directory found: $user_data_dir"
        echo "This directory contains your logs, databases, and other application data."
        echo "It will be preserved during uninstallation."
    fi
}

# Function to update system databases
update_databases() {
    print_info "Updating system databases..."
    
    # Update desktop database
    if command -v update-desktop-database &> /dev/null; then
        sudo update-desktop-database "$DESKTOPDIR" 2>/dev/null || true
    fi
    
    # Update icon cache
    if command -v gtk-update-icon-cache &> /dev/null; then
        sudo gtk-update-icon-cache -f -t "$ICONDIR" 2>/dev/null || true
    fi
    
    print_success "System databases updated"
}

# Function to show help
show_help() {
    echo "Flight Planner Uninstallation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -p, --prefix   Set installation prefix (default: /usr/local)"
    echo "  -y, --yes      Skip confirmation prompt"
    echo ""
    echo "Examples:"
    echo "  $0                    # Uninstall from /usr/local (with confirmation)"
    echo "  $0 --prefix /usr     # Uninstall from /usr"
    echo "  $0 --yes             # Uninstall without confirmation"
    echo ""
}

# Parse command line arguments
SKIP_CONFIRM=false
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -p|--prefix)
            PREFIX="$2"
            BINDIR="$PREFIX/bin"
            DATADIR="$PREFIX/share"
            DESKTOPDIR="$DATADIR/applications"
            ICONDIR="$DATADIR/icons/hicolor"
            shift 2
            ;;
        -y|--yes)
            SKIP_CONFIRM=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main uninstallation process
main() {
    echo "Flight Planner Uninstallation Script"
    echo "===================================="
    echo ""
    
    check_root
    
    if [[ "$SKIP_CONFIRM" != true ]]; then
        confirm_uninstall
    fi
    
    remove_files
    handle_user_data_directory
    update_databases
    
    echo ""
    print_success "Uninstallation complete!"
    echo ""
    print_info "Your application data (~/.local/share/flight-planner/) has been preserved."
    print_info "You can safely remove it manually if no longer needed."
}

# Run main function
main "$@"
