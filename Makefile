# Flight Planner Makefile
# Provides targets for building, installing, and uninstalling the application

# Variables
APP_NAME = flight_planner
APP_ID = com.github.daan.flight-planner
VERSION := $(shell grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Installation directories
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
DESKTOPDIR = $(DATADIR)/applications
ICONDIR = $(DATADIR)/icons/hicolor

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
install:
	./install.sh --prefix $(PREFIX)

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

# Run tests with code coverage
.PHONY: test-coverage
test-coverage: test-coverage-all

.PHONY: test-coverage-all
test-coverage-all:
	cargo tarpaulin --verbose --engine llvm --skip-clean --all-targets --out Lcov --output-dir cov --timeout 120
	mv cov/lcov.info coverage.lcov
	rm -r cov

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
	@	echo "Installation directories (customize with PREFIX):"
	@echo "  Binary:     $(BINDIR)"
	@echo "  Desktop:    $(DESKTOPDIR)"
	@echo "  Icons:      $(ICONDIR)"
	@echo "  App Data:   ~/.local/share/flight-planner/"
