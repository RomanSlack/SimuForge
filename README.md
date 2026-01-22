# SimuForge

![Made with Claude Code](https://img.shields.io/badge/Made%20with-Claude%20Code-orange)

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?logo=webassembly&logoColor=white)
![Rapier](https://img.shields.io/badge/Rapier-Physics-blue)
![Babylon.js](https://img.shields.io/badge/Babylon.js-3D-red)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)

**Physics simulation harness for structured experiments with Claude Code.**

## What is SimuForge?

SimuForge enables Claude to run physics experiments, measure outcomes, and iterate based on structured feedback. It provides:

- **CLI Harness** — Run experiments, compare baselines, evaluate pass/fail criteria
- **Web Visualization** — Real-time 3D rendering of physics simulations
- **Deterministic Physics** — Reproducible results with Rapier's enhanced determinism
- **Structured Reports** — JSON output for automated analysis and iteration

## Claude Feedback Loop

```
1. Receive experiment spec + baseline metrics
2. Analyze physics code + failing metrics
3. Propose code changes
4. Rebuild & run: simuforge run spec.yaml --baseline ref.json
5. Evaluate structured JSON output
6. Iterate or accept
```

## Quick Start

```bash
# Build
cargo build --release
wasm-pack build crates/simuforge-wasm --target web --out-dir ../../packages/simuforge-renderer/pkg

# Run experiment
./target/release/simuforge run experiments/benchmarks/box-stack-10.yaml --pretty

# Start web UI
cd packages/simuforge-web && npm install && npm run dev
```

See [QUICKSTART.md](QUICKSTART.md) for full setup instructions.

## Future Objectives

- [ ] Soft body dynamics and cloth simulation
- [ ] Fluid simulation integration
- [ ] Constraint-based puzzles for reasoning evaluation
- [ ] Multi-agent coordination scenarios
- [ ] Automated hyperparameter tuning via Claude iteration
- [ ] Export to video/GIF for documentation

## License

MIT
