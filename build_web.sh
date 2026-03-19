#!/bin/bash
# Build script for WebAssembly

echo "Building Life's End for WebAssembly..."

# Install wasm-bindgen-cli if not present
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build for wasm32 target
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen --out-dir ./out --target web ./target/wasm32-unknown-unknown/release/lifes_end.wasm

# Copy assets
cp -r assets out/ 2>/dev/null || true
cp index.html out/

echo "Build complete! Serve the 'out' directory with a web server."
echo "Example: python3 -m http.server 8080 --directory out"
