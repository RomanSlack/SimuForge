#!/bin/bash
# Build WASM module for browser use

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building simuforge-wasm..."

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build WASM
cd crates/simuforge-wasm
wasm-pack build --target web --out-dir ../../packages/simuforge-renderer/pkg

echo "WASM build complete!"
echo "Output: packages/simuforge-renderer/pkg/"
