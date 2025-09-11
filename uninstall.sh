#!/bin/bash

# Flight Planner Uninstallation Script
# This script removes the Flight Planner application from both user and system installations

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

# Installation directories
# User installation (default)
USER_BIN="$HOME/.local/bin"
USER_DATADIR="$HOME/.local/share"
USER_DESKTOPDIR="$USER_DATADIR/applications"
USER_ICONDIR="$USER_DATADIR/icons/hicolor"
USER_SHAREAPPDIR="$USER_DATADIR/flight-planner"

# System installation
PREFIX="/usr/local"
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share"
DESKTOPDIR="$DATADIR/applications"
ICONDIR="$DATADIR/icons/hicolor"

# Icon sizes
ICON_SIZES=(16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512)

# Global variables for installation detection
USER_INSTALLED=false
SYSTEM_INSTALLED=false

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

# Helper function to run commands with sudo if needed
run_as_root() {
    if [[ $EUID -eq 0 ]]; then
        "$@"
    else
        sudo "$@"
    fi
}

# Function to detect installation type
detect_installation() {
    # Check for user installation
    if [[ -f "$USER_BIN/$APP_NAME" ]] || [[ -f "$USER_DESKTOPDIR/$APP_ID.desktop" ]]; then
        USER_INSTALLED=true
    fi
    
    # Check for system installation
    if [[ -f "$BINDIR/$APP_NAME" ]] || [[ -f "$DESKTOPDIR/$APP_ID.desktop" ]]; then
        SYSTEM_INSTALLED=true
    fi
}

# Function to confirm uninstallation
confirm_uninstall() {
    echo "Flight Planner installation(s) detected:"
    echo ""
    
    if [[ "$USER_INSTALLED" == "true" ]]; then
        echo "  USER INSTALLATION:"
        echo "    - Binary: $USER_BIN/$APP_NAME"
        echo "    - Desktop file: $USER_DESKTOPDIR/$APP_ID.desktop"
        echo "    - Icons: $USER_ICONDIR/*/apps/$APP_ID.png"
        echo "    - Data: $USER_SHAREAPPDIR/"
        echo ""
    fi
    
    if [[ "$SYSTEM_INSTALLED" == "true" ]]; then
        echo "  SYSTEM INSTALLATION:"
        echo "    - Binary: $BINDIR/$APP_NAME"
        echo "    - Desktop file: $DESKTOPDIR/$APP_ID.desktop"
        echo "    - Icons: $ICONDIR/*/apps/$APP_ID.png"
        echo ""
    fi
    
    if [[ "$USER_INSTALLED" == "false" && "$SYSTEM_INSTALLED" == "false" ]]; then
        print_info "No Flight Planner installation found."
        exit 0
    fi
    
    echo "Your application data (~/.local/share/flight-planner/) will be preserved."
    echo ""
    read -p "Are you sure you want to remove all detected installations? [y/N]: " -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled."
        exit 0
    fi
}

# Function to remove files
remove_files() {
    print_info "Removing application files..."
    
    # Remove user installation
    if [[ "$USER_INSTALLED" == "true" ]]; then
        print_info "Removing user installation..."
        
        # Remove user binary
        if [[ -f "$USER_BIN/$APP_NAME" ]]; then
            rm -f "$USER_BIN/$APP_NAME"
            print_success "User binary removed"
        fi
        
        # Remove user desktop file
        if [[ -f "$USER_DESKTOPDIR/$APP_ID.desktop" ]]; then
            rm -f "$USER_DESKTOPDIR/$APP_ID.desktop"
            print_success "User desktop file removed"
        fi
        
        # Remove user icons
        local user_icons_removed=0
        for size in "${ICON_SIZES[@]}"; do
            if [[ -f "$USER_ICONDIR/$size/apps/$APP_ID.png" ]]; then
                rm -f "$USER_ICONDIR/$size/apps/$APP_ID.png"
                ((user_icons_removed++))
            fi
        done
        
        if [[ $user_icons_removed -gt 0 ]]; then
            print_success "$user_icons_removed user icon files removed"
        fi
    fi
    
    # Remove system installation
    if [[ "$SYSTEM_INSTALLED" == "true" ]]; then
        print_info "Removing system installation..."
        
        # Remove system binary
        if [[ -f "$BINDIR/$APP_NAME" ]]; then
            run_as_root rm -f "$BINDIR/$APP_NAME"
            print_success "System binary removed"
        fi
        
        # Remove system desktop file
        if [[ -f "$DESKTOPDIR/$APP_ID.desktop" ]]; then
            run_as_root rm -f "$DESKTOPDIR/$APP_ID.desktop"
            print_success "System desktop file removed"
        fi
        
        # Remove system icons
        local system_icons_removed=0
        for size in "${ICON_SIZES[@]}"; do
            if [[ -f "$ICONDIR/$size/apps/$APP_ID.png" ]]; then
                run_as_root rm -f "$ICONDIR/$size/apps/$APP_ID.png"
                ((system_icons_removed++))
            fi
        done
        
        if [[ $system_icons_removed -gt 0 ]]; then
            print_success "$system_icons_removed system icon files removed"
        fi
    fi
}

# Function to handle user data directory
handle_user_data_directory() {
    local user_data_dir="$USER_SHAREAPPDIR"
    if [[ -d "$user_data_dir" ]]; then
        echo ""
        print_info "Application data directory found: $user_data_dir"
        echo "This directory contains your logs, databases, and other application data."
        echo ""
        read -p "Do you want to remove your application data as well? [y/N]: " -r
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$user_data_dir"
            print_success "Application data removed"
        else
            print_info "Application data preserved at: $user_data_dir"
        fi
    fi
}

# Function to update system databases
update_databases() {
    print_info "Updating system databases..."
    
    # Update user databases
    if [[ "$USER_INSTALLED" == "true" ]]; then
        # Update user desktop database
        if command -v update-desktop-database &> /dev/null; then
            update-desktop-database "$USER_DESKTOPDIR" 2>/dev/null || true
        fi
        
        # Update user icon cache
        if command -v gtk-update-icon-cache &> /dev/null; then
            gtk-update-icon-cache -f -t "$USER_ICONDIR" 2>/dev/null || true
        fi
    fi
    
    # Update system databases
    if [[ "$SYSTEM_INSTALLED" == "true" ]]; then
        # Update system desktop database
        if command -v update-desktop-database &> /dev/null; then
            run_as_root update-desktop-database "$DESKTOPDIR" 2>/dev/null || true
        fi
        
        # Update system icon cache
        if command -v gtk-update-icon-cache &> /dev/null; then
            run_as_root gtk-update-icon-cache -f -t "$ICONDIR" 2>/dev/null || true
        fi
    fi
    
    print_success "System databases updated"
}

# Function to show help
show_help() {
    echo "Flight Planner Uninstallation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "This script automatically detects and removes both user and system installations."
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -y, --yes      Skip confirmation prompts"
    echo ""
    echo "Installation Detection:"
    echo "  - User installation: ~/.local/bin, ~/.local/share/applications, etc."
    echo "  - System installation: /usr/local/bin, /usr/local/share/applications, etc."
    echo ""
    echo "Examples:"
    echo "  $0                    # Remove all detected installations (with confirmation)"
    echo "  $0 --yes             # Remove all detected installations without confirmation"
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
    
    # Detect installation type once
    detect_installation
    
    if [[ "$SKIP_CONFIRM" != true ]]; then
        confirm_uninstall
    fi
    
    remove_files
    handle_user_data_directory
    update_databases
    
    echo ""
    print_success "Uninstallation complete!"
    echo ""
    
    # Check if user data still exists
    if [[ -d "$USER_SHAREAPPDIR" ]]; then
        print_info "Your application data has been preserved at: $USER_SHAREAPPDIR"
        print_info "You can safely remove it manually if no longer needed."
    fi
}

# Run main function
main "$@"
