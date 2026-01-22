#!/bin/bash
# Run all benchmark experiments

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building simuforge-harness..."
cargo build --release -p simuforge-harness

echo ""
echo "Running benchmark suite..."
./target/release/simuforge suite experiments/benchmarks/ -o results/

echo ""
echo "Benchmark run complete!"
