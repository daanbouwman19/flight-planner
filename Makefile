# Flight Planner Makefile
# Provides targets for building, installing, and uninstalling the application

# Variables
APP_NAME = flight_planner
APP_ID = com.github.daan.flight-planner
VERSION = 0.1.0

# Installation directories
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
DESKTOPDIR = $(DATADIR)/applications
ICONDIR = $(DATADIR)/icons/hicolor
LOGDIR = /var/log/$(APP_NAME)

# Icon sizes for installation
ICON_SIZES = 16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512

# Default target
.PHONY: all
all: build

# Build the application
.PHONY: build
build:
	cargo build --release

# Install the application
.PHONY: install
install: build
	@echo "Installing Flight Planner..."
	
	# Create directories
	sudo mkdir -p $(BINDIR)
	sudo mkdir -p $(DESKTOPDIR)
	sudo mkdir -p $(ICONDIR)
	sudo mkdir -p $(LOGDIR)
	sudo chown $(USER):$(USER) $(LOGDIR)
	
	# Install binary
	sudo cp target/release/$(APP_NAME) $(BINDIR)/
	sudo chmod +x $(BINDIR)/$(APP_NAME)
	
	# Install desktop file
	sudo cp $(APP_ID).desktop $(DESKTOPDIR)/
	
	# Install icon (create symlinks for different sizes)
	@for size in $(ICON_SIZES); do \
		sudo mkdir -p $(ICONDIR)/$$size/apps; \
		sudo cp icon.png $(ICONDIR)/$$size/apps/$(APP_ID).png; \
	done
	
	# Update desktop database
	sudo update-desktop-database $(DESKTOPDIR) 2>/dev/null || true
	
	# Update icon cache
	sudo gtk-update-icon-cache -f -t $(ICONDIR) 2>/dev/null || true
	
	@echo ""
	@echo "Installation complete!"
	@echo ""
	@echo "IMPORTANT: You need to provide your own airports database:"
	@echo "Application data directory: $$HOME/.local/share/flight-planner/"
	@echo ""
	@echo "1. Place your airports.db3 file in the application data directory (recommended):"
	@echo "   cp /path/to/your/airports.db3 $$HOME/.local/share/flight-planner/airports.db3"
	@echo "2. Or run the application from the directory containing airports.db3"
	@echo ""
	@echo "The application will create its own data.db file for aircraft and flight history"
	@echo "in: $$HOME/.local/share/flight-planner/"
	@echo "Logs are stored in: $$HOME/.local/share/flight-planner/logs/"

# Uninstall the application
.PHONY: uninstall
uninstall:
	@echo "Uninstalling Flight Planner..."
	
	# Remove binary
	sudo rm -f $(BINDIR)/$(APP_NAME)
	
	# Remove desktop file
	sudo rm -f $(DESKTOPDIR)/$(APP_ID).desktop
	
	# Remove icons
	@for size in $(ICON_SIZES); do \
		sudo rm -f $(ICONDIR)/$$size/apps/$(APP_ID).png; \
	done
	
	# Remove log directory (ask user first)
	@echo "Remove log directory $(LOGDIR)? [y/N]"
	@read -r response; \
	if [ "$$response" = "y" ] || [ "$$response" = "Y" ]; then \
		sudo rm -rf $(LOGDIR); \
	fi
	
	# Update desktop database
	sudo update-desktop-database $(DESKTOPDIR) 2>/dev/null || true
	
	# Update icon cache
	sudo gtk-update-icon-cache -f -t $(ICONDIR) 2>/dev/null || true
	
	@echo "Uninstallation complete!"

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Run the application (for development)
.PHONY: run
run:
	cargo run

# Run tests
.PHONY: test
test:
	cargo test

# Check code formatting
.PHONY: fmt
fmt:
	cargo fmt

# Run clippy lints
.PHONY: clippy
clippy:
	cargo clippy

# Show help
.PHONY: help
help:
	@echo "Flight Planner Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build      - Build the application in release mode"
	@echo "  install    - Install the application system-wide"
	@echo "  uninstall  - Remove the application from the system"
	@echo "  clean      - Clean build artifacts"
	@echo "  run        - Run the application in development mode"
	@echo "  test       - Run tests"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run clippy lints"
	@echo "  help       - Show this help message"
	@echo ""
	@echo "Installation directories (customize with PREFIX):"
	@echo "  Binary:     $(BINDIR)"
	@echo "  Desktop:    $(DESKTOPDIR)"
	@echo "  Icons:      $(ICONDIR)"
	@echo "  Logs:       $(LOGDIR)"
