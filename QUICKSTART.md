# SimuForge Quick Start

## Prerequisites
```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack (one-time)
cargo install wasm-pack
```

## First-Time Setup
```bash
cd ~/SimuForge

# Build Rust CLI
cargo build --release

# Build WASM for web
wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg

# Install Node dependencies
cd packages/simuforge-web
npm install
```

## Start Web UI
```bash
cd ~/SimuForge/packages/simuforge-web
npm run dev
```
Opens at http://localhost:3000

## CLI Usage

```bash
# Run an experiment
./target/release/simuforge run experiments/benchmarks/box-stack-10.yaml --pretty

# Compare to baseline
./target/release/simuforge run experiments/benchmarks/box-stack-10.yaml --baseline experiments/baselines/box-stack-10.json

# Generate a baseline
./target/release/simuforge baseline experiments/benchmarks/box-stack-10.yaml -o experiments/baselines/box-stack-10.json

# Run full benchmark suite
./target/release/simuforge suite experiments/benchmarks/ -o results/

# Validate experiment file
./target/release/simuforge validate experiments/benchmarks/box-stack-10.yaml

# List scenarios
./target/release/simuforge scenarios
```

## After Code Changes

### Changed Rust code:
```bash
cargo build --release
```

### Changed WASM code (crates/simuforge-wasm):
```bash
wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg
```
Then refresh browser.

### Changed web code:
Vite auto-reloads.

## One-liner: Full Rebuild
```bash
cd ~/SimuForge && cargo build --release && wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg
```

## Project Structure
```
SimuForge/
├── crates/
│   ├── simuforge-core/      # Core types
│   ├── simuforge-physics/   # Rapier physics
│   ├── simuforge-harness/   # CLI tool
│   └── simuforge-wasm/      # Browser bindings
├── packages/
│   └── simuforge-web/       # Web UI (Vite + Babylon.js)
├── experiments/
│   ├── benchmarks/          # Test scenarios
│   └── baselines/           # Reference results
└── target/release/simuforge # CLI binary
```
