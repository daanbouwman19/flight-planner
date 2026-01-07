#
# Flight Planner Makefile
#
# This Makefile provides a set of targets to simplify common development tasks
# such as building, testing, and installing the Flight Planner application.
#

# ==============================================================================
# Variables
# ==============================================================================

# The name of the application binary.
APP_NAME = flight_planner
# The application's unique identifier, used for desktop integration.
APP_ID = com.github.daan.flight-planner
# The application version, extracted from Cargo.toml.
VERSION := $(shell grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# ==============================================================================
# Installation Directories
# ==============================================================================

# The base directory for installation. Can be overridden by the user.
PREFIX ?= /usr/local
# The directory where the application binary will be installed.
BINDIR = $(PREFIX)/bin
# The base directory for shared data files.
DATADIR = $(PREFIX)/share
# The directory for the .desktop file, for application menu integration.
DESKTOPDIR = $(DATADIR)/applications
# The base directory for application icons.
ICONDIR = $(DATADIR)/icons/hicolor

# A list of icon sizes to be installed.
ICON_SIZES = 16x16 22x22 24x24 32x32 48x48 64x64 128x128 256x256 512x512

# ==============================================================================
# Core Targets
# ==============================================================================

# The default target, which builds the application.
.PHONY: all
all: build

# Build the application in release mode for optimal performance.
.PHONY: build
build:
	cargo build --release

# Install the application system-wide using the install.sh script.
# The PREFIX variable can be used to customize the installation location.
.PHONY: install
install:
	./install.sh --prefix $(PREFIX)

# Uninstall the application from the system.
# This target removes the binary, .desktop file, and icons.
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
	
	
	# Update desktop database to unregister the application.
	sudo update-desktop-database $(DESKTOPDIR) 2>/dev/null || true
	
	# Update icon cache to remove the application's icons.
	sudo gtk-update-icon-cache -f -t $(ICONDIR) 2>/dev/null || true
	
	@echo "Uninstallation complete!"

# Clean build artifacts to free up space and ensure a fresh build.
.PHONY: clean
clean:
	cargo clean

# Run the application in development mode with debug symbols.
.PHONY: run
run:
	cargo run

# ==============================================================================
# Quality Assurance Targets
# ==============================================================================

# Run the test suite.
.PHONY: test
test:
	cargo test

# Generate a code coverage report using cargo-tarpaulin.
# The --all-targets flag ensures that all code is included in the report.
.PHONY: test-coverage
test-coverage:
	cargo tarpaulin --all-targets --jobs 1 --out Lcov --output-dir cov --fail-under 80
	lcov --add-tracefile cov/lcov.info --output-file coverage.lcov

# Check code formatting against the project's style guidelines.
.PHONY: fmt
fmt:
	cargo fmt

# Run clippy to check for common mistakes and code style issues.
.PHONY: clippy
clippy:
	cargo clippy

# ==============================================================================
# Help
# ==============================================================================

# Show a help message that lists and describes the available targets.
.PHONY: help
help:
	@echo "Flight Planner Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build the application in release mode"
	@echo "  install        - Install the application system-wide"
	@echo "  uninstall      - Remove the application from the system"
	@echo "  clean          - Clean build artifacts"
	@echo "  run            - Run the application in development mode"
	@echo "  test           - Run tests"
	@echo "  test-coverage  - Generate a code coverage report"
	@echo "  fmt            - Format code"
	@echo "  clippy         - Run clippy lints"
	@echo "  help           - Show this help message"
	@echo ""
	@	echo "Installation directories (customize with PREFIX):"
	@echo "  Binary:     $(BINDIR)"
	@echo "  Desktop:    $(DESKTOPDIR)"
	@echo "  Icons:      $(ICONDIR)"
	@echo "  App Data:   ~/.local/share/flight-planner/"
