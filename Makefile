# Makefile for ailater-im
# AI-powered input method for fcitx5

# Installation prefix
PREFIX ?= /usr
LIB_DIR = $(PREFIX)/lib/x86_64-linux-gnu/fcitx5
DATA_DIR = $(PREFIX)/share/ailater-im
ADDON_DIR = $(PREFIX)/share/fcitx5/addon
INPUTMETHOD_DIR = $(PREFIX)/share/fcitx5/inputmethod
ICON_DIR = $(PREFIX)/share/icons/hicolor
ICON_SRC_DIR = src/img/icons
BIN_DIR = $(PREFIX)/bin
CONFIG_TOOL_DIR = config-tool

# Rust target
TARGET ?= release
CARGO_FLAGS = --$(TARGET)

# Features
FEATURES ?= fcitx5,remote-model

.PHONY: all build install uninstall uninstall-user clean test doc config-tool

all: build config-tool

build:
	@echo "Building ailater-im..."
	cargo build $(CARGO_FLAGS) --features "$(FEATURES)"

config-tool:
	@echo "Building config tool..."
	cd $(CONFIG_TOOL_DIR) && cargo build --release

install: build config-tool
	@echo "Installing to $(PREFIX)..."
	install -d $(DESTDIR)$(LIB_DIR)
	install -d $(DESTDIR)$(DATA_DIR)/dict
	install -d $(DESTDIR)$(ADDON_DIR)
	install -d $(DESTDIR)$(INPUTMETHOD_DIR)
	install -d $(DESTDIR)$(BIN_DIR)
	install -m 755 target/$(TARGET)/libailater_im.so $(DESTDIR)$(LIB_DIR)/
	install -m 644 conf/ailater-im.conf $(DESTDIR)$(ADDON_DIR)/
	install -m 644 conf/inputmethod/ailater-im.conf $(DESTDIR)$(INPUTMETHOD_DIR)/
	install -m 644 data/system.dict $(DESTDIR)$(DATA_DIR)/dict/
	install -m 644 data/config.toml $(DESTDIR)$(DATA_DIR)/
	install -m 755 $(CONFIG_TOOL_DIR)/target/release/ailater-config $(DESTDIR)$(BIN_DIR)/ailater-config
	@echo "Installing app icons..."
	for size in 16 22 24 48; do \
		install -d $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/apps; \
		install -m 644 $(ICON_SRC_DIR)/$${size}x$${size}/apps/ailater-im.png $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/apps/org.fcitx.Fcitx5.ailater-im.png; \
	done
	install -d $(DESTDIR)$(ICON_DIR)/scalable/apps
	install -m 644 $(ICON_SRC_DIR)/scalable/apps/ailater-im.svg $(DESTDIR)$(ICON_DIR)/scalable/apps/org.fcitx.Fcitx5.ailater-im.svg
	@echo "Installing tray icons..."
	for size in 16 22 24; do \
		install -d $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/status; \
		install -m 644 $(ICON_SRC_DIR)/status/$${size}/fcitx-ailater.svg $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/status/; \
		install -m 644 $(ICON_SRC_DIR)/status/$${size}/fcitx-ailater-dark.svg $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/status/; \
	done
	@echo "Installing desktop entry..."
	install -d $(DESTDIR)$(PREFIX)/share/applications
	install -m 644 $(CONFIG_TOOL_DIR)/ailater-config.desktop $(DESTDIR)$(PREFIX)/share/applications/
	@echo "Updating icon cache..."
	gtk-update-icon-cache $(DESTDIR)$(ICON_DIR) 2>/dev/null || true

uninstall:
	@echo "Uninstalling from $(PREFIX)..."
	rm -f $(DESTDIR)$(LIB_DIR)/libailater_im.so
	rm -f $(DESTDIR)$(ADDON_DIR)/ailater-im.conf
	rm -f $(DESTDIR)$(INPUTMETHOD_DIR)/ailater-im.conf
	rm -f $(DESTDIR)$(BIN_DIR)/ailater-config
	rm -f $(DESTDIR)$(PREFIX)/share/applications/ailater-config.desktop
	rm -rf $(DESTDIR)$(DATA_DIR)
	@echo "Removing icons..."
	for size in 16 22 24 48; do \
		rm -f $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/apps/org.fcitx.Fcitx5.ailater-im.png; \
	done
	rm -f $(DESTDIR)$(ICON_DIR)/scalable/apps/org.fcitx.Fcitx5.ailater-im.svg
	for size in 16 22 24; do \
		rm -f $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/status/fcitx-ailater.svg; \
		rm -f $(DESTDIR)$(ICON_DIR)/$${size}x$${size}/status/fcitx-ailater-dark.svg; \
	done
	@echo "Updating icon cache..."
	gtk-update-icon-cache $(DESTDIR)$(ICON_DIR) 2>/dev/null || true

clean:
	cargo clean
	cd $(CONFIG_TOOL_DIR) && cargo clean 2>/dev/null || true

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
	install -d ~/.local/lib/x86_64-linux-gnu/fcitx5
	install -d ~/.local/share/ailater-im/dict
	install -d ~/.local/share/fcitx5/addon
	install -d ~/.local/share/fcitx5/inputmethod
	install -m 755 target/$(TARGET)/libailater_im.so ~/.local/lib/x86_64-linux-gnu/fcitx5/
	install -m 644 conf/ailater-im.conf ~/.local/share/fcitx5/addon/
	install -m 644 conf/inputmethod/ailater-im.conf ~/.local/share/fcitx5/inputmethod/
	install -m 644 data/system.dict ~/.local/share/ailater-im/dict/
	install -m 644 data/config.toml ~/.local/share/ailater-im/
	@echo "Installing app icons..."
	for size in 16 22 24 48; do \
		install -d ~/.local/share/icons/hicolor/$${size}x$${size}/apps; \
		install -m 644 $(ICON_SRC_DIR)/$${size}x$${size}/apps/ailater-im.png ~/.local/share/icons/hicolor/$${size}x$${size}/apps/org.fcitx.Fcitx5.ailater-im.png; \
	done
	install -d ~/.local/share/icons/hicolor/scalable/apps
	install -m 644 $(ICON_SRC_DIR)/scalable/apps/ailater-im.svg ~/.local/share/icons/hicolor/scalable/apps/org.fcitx.Fcitx5.ailater-im.svg
	@echo "Installing tray icons..."
	for size in 16 22 24; do \
		install -d ~/.local/share/icons/hicolor/$${size}x$${size}/status; \
		install -m 644 $(ICON_SRC_DIR)/status/$${size}/fcitx-ailater.svg ~/.local/share/icons/hicolor/$${size}x$${size}/status/; \
		install -m 644 $(ICON_SRC_DIR)/status/$${size}/fcitx-ailater-dark.svg ~/.local/share/icons/hicolor/$${size}x$${size}/status/; \
	done
	@echo "Updating icon cache..."
	gtk-update-icon-cache ~/.local/share/icons/hicolor 2>/dev/null || true

# Uninstall from user directory
uninstall-user:
	@echo "Uninstalling from user directory..."
	rm -f ~/.local/lib/x86_64-linux-gnu/fcitx5/libailater_im.so
	rm -f ~/.local/share/fcitx5/addon/ailater-im.conf
	rm -f ~/.local/share/fcitx5/inputmethod/ailater-im.conf
	rm -rf ~/.local/share/ailater-im
	@echo "Removing icons..."
	for size in 16 22 24 48; do \
		rm -f ~/.local/share/icons/hicolor/$${size}x$${size}/apps/org.fcitx.Fcitx5.ailater-im.png; \
	done
	rm -f ~/.local/share/icons/hicolor/scalable/apps/org.fcitx.Fcitx5.ailater-im.svg
	for size in 16 22 24; do \
		rm -f ~/.local/share/icons/hicolor/$${size}x$${size}/status/fcitx-ailater.svg; \
		rm -f ~/.local/share/icons/hicolor/$${size}x$${size}/status/fcitx-ailater-dark.svg; \
	done
	@echo "Updating icon cache..."
	gtk-update-icon-cache ~/.local/share/icons/hicolor 2>/dev/null || true

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
	@echo "  uninstall-user - Remove from user directory"
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
