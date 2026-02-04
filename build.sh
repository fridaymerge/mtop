#!/bin/bash
# Build script for mtop Rust version

set -e

echo "🦀 Building mtop (Rust version)..."
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "📦 Building release binary..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "Binary location: ./target/release/mtop"
    echo "Binary size: $(du -h target/release/mtop | cut -f1)"
    echo ""
    echo "Run it with:"
    echo "  ./target/release/mtop"
    echo ""
    echo "Or install globally with:"
    echo "  cargo install --path ."
else
    echo ""
    echo "❌ Build failed"
    exit 1
fi
