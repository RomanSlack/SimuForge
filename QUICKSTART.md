# SimuForge Quick Start

## Prerequisites
```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack (one-time)
cargo install wasm-pack

# Install Node.js dependencies (one-time, from project root)
cd ~/SimuForge
npm install
```

## Build Commands

### Build Rust CLI (release)
```bash
cargo build --release
```

### Build WASM for web
```bash
wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg
```

### Start web dev server
```bash
cd packages/simuforge-web
npm run dev
```

## CLI Usage

### Run an experiment
```bash
./target/release/simuforge run experiments/benchmarks/box-stack-10.yaml --pretty
```

### Compare to baseline
```bash
./target/release/simuforge run experiments/benchmarks/box-stack-10.yaml --baseline experiments/baselines/box-stack-10.json
```

### Generate a baseline
```bash
./target/release/simuforge baseline experiments/benchmarks/box-stack-10.yaml -o experiments/baselines/box-stack-10.json
```

### Run full benchmark suite
```bash
./target/release/simuforge suite experiments/benchmarks/ -o results/
```

### Validate experiment file
```bash
./target/release/simuforge validate experiments/benchmarks/box-stack-10.yaml
```

### List scenarios
```bash
./target/release/simuforge scenarios
```

## Full Rebuild (after pulling changes)
```bash
cd ~/SimuForge
cargo build --release
wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg
cd packages/simuforge-web && npm run dev
```

## One-liner: Build everything
```bash
cargo build --release && wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg
```
