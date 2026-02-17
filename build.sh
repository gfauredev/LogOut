#!/bin/bash
# Build script for LogOut workout app

set -e

echo "Building LogOut for Web..."

# Build the wasm binary
echo "Step 1: Building WASM binary..."
cargo build --target wasm32-unknown-unknown --release

# Generate wasm bindings
echo "Step 2: Generating WASM bindings..."
mkdir -p dist
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/release/logout.wasm

echo "Build complete! Files are in the 'dist' directory."
echo ""
echo "To run the app locally:"
echo "  python3 -m http.server 8080"
echo "  Then open http://localhost:8080 in your browser"
