#!/bin/bash
# Install Rust toolchain for building fcitx5-ai-im

set -e

echo "Installing Rust toolchain..."

# Check if rustup is already installed
if command -v rustup &> /dev/null; then
    echo "Rustup is already installed"
    rustup --version
    exit 0
fi

# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add cargo to path
source "$HOME/.cargo/env"

# Verify installation
echo ""
echo "Rust installation complete!"
echo "  rustc: $(rustc --version)"
echo "  cargo: $(cargo --version)"
echo ""
echo "To use Rust in the current terminal, run:"
echo "  source \$HOME/.cargo/env"
echo ""
echo "Then build the project with:"
echo "  make build"
