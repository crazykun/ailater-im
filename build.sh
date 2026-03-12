#!/bin/bash
# Build script for ailater-im

set -e

echo "Building ailater-im..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust."
    exit 1
fi

# Build the library
cargo build --release

# Create output directories
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr}"
LIB_DIR="${INSTALL_PREFIX}/lib/fcitx5"
DATA_DIR="${INSTALL_PREFIX}/share/ailater-im"
CONFIG_DIR="${INSTALL_PREFIX}/share/fcitx5/config-addon"

echo "Installing to ${INSTALL_PREFIX}..."

# Create directories
sudo mkdir -p "${LIB_DIR}"
sudo mkdir -p "${DATA_DIR}/dict"
sudo mkdir -p "${CONFIG_DIR}"

# Install the library
sudo install -m 755 target/release/libfcitx5_ai_im.so "${LIB_DIR}/"

# Install configuration
sudo install -m 644 conf/ailater-im.conf "${CONFIG_DIR}/"

# Install dictionary (if exists)
if [ -f "data/system.dict" ]; then
    sudo install -m 644 data/system.dict "${DATA_DIR}/dict/"
fi

echo "Installation complete!"
echo ""
echo "To use this input method:"
echo "1. Restart fcitx5: fcitx5 -r"
echo "2. Add 'AI Pinyin' in fcitx5 configuration"
echo "3. Configure the AI model endpoint in ~/.config/ailater-im/config.toml"
