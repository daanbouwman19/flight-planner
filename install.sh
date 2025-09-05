#!/bin/bash

# Flight Planner Installation Script
# This script installs the Flight Planner application system-wide

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
VERSION="0.1.0"

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

# Function to check dependencies
check_dependencies() {
    print_info "Checking dependencies..."
    
    local missing_deps=()
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("rust/cargo")
    fi
    
    if ! command -v sudo &> /dev/null; then
        missing_deps+=("sudo")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        print_info "Please install the missing dependencies and try again."
        exit 1
    fi
    
    print_success "All dependencies found"
}

# Function to build the application
build_app() {
    print_info "Building Flight Planner..."
    
    if ! cargo build --release; then
        print_error "Build failed!"
        exit 1
    fi
    
    print_success "Build completed"
}

# Function to create directories
create_directories() {
    print_info "Creating installation directories..."
    
    sudo mkdir -p "$BINDIR"
    sudo mkdir -p "$DESKTOPDIR"
    sudo mkdir -p "$ICONDIR"
    
    print_success "Directories created"
}

# Function to install files
install_files() {
    print_info "Installing application files..."
    
    # Install binary
    sudo cp "target/release/$APP_NAME" "$BINDIR/"
    sudo chmod +x "$BINDIR/$APP_NAME"
    
    # Install desktop file
    sudo cp "$APP_ID.desktop" "$DESKTOPDIR/"
    
    # Install icons for different sizes
    for size in "${ICON_SIZES[@]}"; do
        sudo mkdir -p "$ICONDIR/$size/apps"
        sudo cp icon.png "$ICONDIR/$size/apps/$APP_ID.png"
    done
    
    print_success "Files installed"
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

# Function to show post-installation instructions
show_instructions() {
    echo ""
    print_success "Installation complete!"
    echo ""
    print_warning "IMPORTANT: You need to provide your own airports database!"
    echo ""
    echo "The Flight Planner requires an airports database file (airports.db3) to function."
    echo ""
    echo "Application data directory: $HOME/.local/share/flight-planner/"
    echo ""
    echo "Option 1: Place airports.db3 in the application data directory (recommended)"
    echo "  cp /path/to/your/airports.db3 $HOME/.local/share/flight-planner/airports.db3"
    echo ""
    echo "Option 2: Run the application from the directory containing airports.db3"
    echo "  cd /path/to/directory/with/airports.db3"
    echo "  $APP_NAME"
    echo ""
    echo "The application will automatically create its own data.db file for"
    echo "aircraft and flight history in: $HOME/.local/share/flight-planner/"
    echo ""
    echo "Logs are stored in: $HOME/.local/share/flight-planner/logs/"
    echo ""
    echo "You can now launch Flight Planner from your application menu!"
}

# Function to show help
show_help() {
    echo "Flight Planner Installation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -p, --prefix   Set installation prefix (default: /usr/local)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Install to /usr/local"
    echo "  $0 --prefix /usr     # Install to /usr"
    echo ""
}

# Parse command line arguments
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
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main installation process
main() {
    echo "Flight Planner Installation Script v$VERSION"
    echo "=============================================="
    echo ""
    
    check_root
    check_dependencies
    build_app
    create_directories
    install_files
    update_databases
    show_instructions
}

# Run main function
main "$@"
