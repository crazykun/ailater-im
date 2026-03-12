# Makefile for ailater-im
# AI-powered input method for fcitx5

# Installation prefix
PREFIX ?= /usr
LIB_DIR = $(PREFIX)/lib/fcitx5
DATA_DIR = $(PREFIX)/share/ailater-im
ADDON_DIR = $(PREFIX)/share/fcitx5/addon

# Rust target
TARGET ?= release
CARGO_FLAGS = --$(TARGET)

# Features
FEATURES ?= remote-model

.PHONY: all build install uninstall clean test doc

all: build

build:
	@echo "Building ailater-im..."
	cargo build $(CARGO_FLAGS) --features "$(FEATURES)"

install: build
	@echo "Installing to $(PREFIX)..."
	install -d $(DESTDIR)$(LIB_DIR)
	install -d $(DESTDIR)$(DATA_DIR)/dict
	install -d $(DESTDIR)$(ADDON_DIR)
	install -m 755 target/$(TARGET)/libailater_im.so $(DESTDIR)$(LIB_DIR)/
	install -m 644 conf/ailater-im.conf $(DESTDIR)$(ADDON_DIR)/
	install -m 644 data/system.dict $(DESTDIR)$(DATA_DIR)/dict/
	install -m 644 data/config.toml $(DESTDIR)$(DATA_DIR)/

uninstall:
	@echo "Uninstalling from $(PREFIX)..."
	rm -f $(DESTDIR)$(LIB_DIR)/libailater_im.so
	rm -f $(DESTDIR)$(ADDON_DIR)/ailater-im.conf
	rm -rf $(DESTDIR)$(DATA_DIR)

clean:
	cargo clean

test:
	cargo test --all-features

doc:
	cargo doc --no-deps --all-features

# Development targets
dev:
	cargo build --features "remote-model"

dev-local:
	cargo build --features "local-model"

dev-full:
	cargo build --features "full"

# Release build with optimizations
release:
	cargo build --release --features "$(FEATURES)"

# Check code without building
check:
	cargo check --all-features

# Format code
fmt:
	cargo fmt

# Run linter
lint:
	cargo clippy --all-features -- -D warnings

# Install to user directory
install-user: build
	@echo "Installing to user directory..."
	install -d ~/.local/lib/fcitx5
	install -d ~/.local/share/ailater-im/dict
	install -d ~/.local/share/fcitx5/addon
	install -m 755 target/$(TARGET)/libailater_im.so ~/.local/lib/fcitx5/
	install -m 644 conf/ailater-im.conf ~/.local/share/fcitx5/addon/
	install -m 644 data/system.dict ~/.local/share/ailater-im/dict/
	install -m 644 data/config.toml ~/.local/share/ailater-im/

# Create distribution archive
dist: release
	mkdir -p dist
	cp target/release/libailater_im.so dist/
	cp conf/ailater-im.conf dist/
	cp data/system.dict dist/
	cp data/config.toml dist/
	tar -czvf ailater-im-$(shell date +%Y%m%d).tar.gz dist/
	rm -rf dist

help:
	@echo "ailater-im Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build the project (default)"
	@echo "  build        - Build the project"
	@echo "  install      - Install to system (requires root)"
	@echo "  install-user - Install to user directory"
	@echo "  uninstall    - Remove from system"
	@echo "  clean        - Clean build artifacts"
	@echo "  test         - Run tests"
	@echo "  doc          - Generate documentation"
	@echo "  dev          - Development build"
	@echo "  release      - Optimized release build"
	@echo "  check        - Check code without building"
	@echo "  fmt          - Format code"
	@echo "  lint         - Run clippy linter"
	@echo "  dist         - Create distribution archive"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX       - Installation prefix (default: /usr)"
	@echo "  FEATURES     - Cargo features (default: remote-model)"
	@echo "  TARGET       - Build target (default: release)"
